use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{tag, take_until, take_while1},
        take,
    },
    character::complete::{
        alpha1, alphanumeric1, char, i32 as parse_i32, u8 as parse_u8, usize as parse_usize,
    },
    combinator::{map, recognize},
    error::{Error, ErrorKind},
    multi::many0,
    sequence::{preceded, tuple},
};

use crate::commands::{
    CompressionMethod, CompressionType, GraficData, Orientation, ZplFormatCommand,
};

pub fn parse_pw(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^PW")(input)?;
    let (input, width) = parse_usize(input)?;
    Ok((input, ZplFormatCommand::PrintWidth(width)))
}

pub fn parse_ll(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^LL")(input)?;
    let (input, length) = parse_usize(input)?;
    Ok((input, ZplFormatCommand::LabelLength(length)))
}

pub fn parse_ls(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^LS")(input)?;
    let (input, length) = parse_i32(input)?;
    Ok((input, ZplFormatCommand::LabelShift(length)))
}

pub fn parse_a(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^A")(input)?;

    let (input, (font, orientation, _, height, _, width)) = tuple((
        take(1u8),
        take(1u8),
        char(','),
        parse_usize,
        char(','),
        parse_usize,
    ))(input)?;

    // let (input, font) = take(1u8).parse(input)?;
    let font = font.chars().next().unwrap_or('A');
    // let (input, orientation) = take(1u8).parse(input)?;
    let orientation = match orientation {
        "N" => Orientation::Normal,
        "R" => Orientation::Rotate,
        "I" => Orientation::Invert,
        "B" => Orientation::BackRotate,
        _ => return IResult::Err(nom::Err::Error(Error::new(input, ErrorKind::NoneOf))),
    };
    Ok((
        input,
        ZplFormatCommand::Font {
            name: font,
            orientation,
            height,
            width,
        },
    ))
}

fn parse_coordinates(input: &str) -> IResult<&str, (i32, i32, Option<u8>)> {
    alt((
        map(
            tuple((parse_i32, char(','), parse_i32, char(','), parse_u8)),
            |(x, _, y, _, z)| (x, y, Some(z)),
        ),
        map(tuple((parse_i32, char(','), parse_i32)), |(x, _, y)| {
            (x, y, None)
        }),
    ))
    .parse(input)
}

pub fn parse_fo(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, (x, y, justification)) = preceded(tag("^FO"), parse_coordinates).parse(input)?;
    let justification = match justification {
        Some(u) => u,
        None => 0,
    };

    Ok((
        input,
        ZplFormatCommand::FieldOrigin {
            x,
            y,
            justification,
        },
    ))
}

pub fn parse_ft(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, (x, y, justification)) = preceded(tag("^FT"), parse_coordinates).parse(input)?;
    let justification = match justification {
        Some(u) => u,
        None => 0,
    };

    Ok((
        input,
        ZplFormatCommand::FieldTypeset {
            x,
            y,
            justification,
        },
    ))
}

pub fn parse_fd(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^FD")(input)?;
    let (input, text) = take_until("^FS")(input)?;
    Ok((input, ZplFormatCommand::FieldData(text.to_string())))
}

pub fn parse_fs(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^FS")(input)?;
    Ok((input, ZplFormatCommand::FieldSeparator))
}

pub fn parse_fg(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^GF")(input)?;
    let (input, (compression_type, _, data_bytes, _, total_bytes, _, row_bytes, _)) = tuple((
        alpha1,
        char(','),
        parse_usize,
        char(','),
        parse_usize,
        char(','),
        parse_usize,
        char(','),
    ))(input)?;

    let compression_type = match compression_type {
        "A" => CompressionType::Ascii,
        "B" => CompressionType::Binary,
        "C" => CompressionType::Compressed,
        _ => return IResult::Err(nom::Err::Error(Error::new(input, ErrorKind::NoneOf))),
    };

    let (input, (_, compression_method, _, img_data)) =
        tuple((char(':'), alphanumeric1, char(':'), take(data_bytes)))(input)?;
    let compression_method = match compression_method {
        "Z64" => CompressionMethod::Zlib,
        _ => return IResult::Err(nom::Err::Error(Error::new(input, ErrorKind::NoneOf))),
    };
    let data = GraficData {
        compression_method,
        data: img_data.into(),
    };

    Ok((
        input,
        ZplFormatCommand::GraficField {
            compression_type,
            data_bytes,
            total_bytes,
            row_bytes,
            data,
        },
    ))
}

fn parse_command_name(input: &str) -> IResult<&str, &str> {
    // Matches e.g. "^FO", "^FD", "~DG", "^XYZ"
    recognize(preceded(
        nom::character::complete::one_of("^~"),
        take_while1(|c: char| c.is_ascii_uppercase()),
    ))
    .parse(input)
}

// fn skip_unknown_command(input: &str) -> IResult<&str, ()> {
//     // 1. Consume the command name
//     let (input, _) = parse_command_name(input)?;

//     // 2. Consume until next command or end of file
//     let (remaining, _) = nom::combinator::opt(take_until("^"))
//         .parse(input)
//         .map(|(r, _)| (r, ()))?;

//     Ok((remaining, ()))
// }
fn skip_unknown_command(input: &str) -> IResult<&str, ()> {
    // 1. Consume the command name
    let (input, _) = parse_command_name(input)?;

    // 2. Consume everything until next command or end
    let (remaining, _) = nom::bytes::complete::take_till(|c| c == '^' || c == '~')
        .parse(input)
        .map(|(r, _)| (r, ()))?;

    Ok((remaining, ()))
}

pub fn parse_command(input: &str) -> IResult<&str, ZplFormatCommand> {
    alt((
        map(parse_fo, |c| c),
        map(parse_fd, |c| c),
        map(parse_a, |c| c),
        map(parse_fg, |c| c),
        map(parse_ft, |c| c),
        map(parse_ll, |c| c),
        map(parse_ls, |c| c),
        map(parse_pw, |c| c),
        map(parse_fs, |c| c),
        // add more commands here
    ))
    .parse(input)
}

pub fn skip_until_command(input: &str) -> IResult<&str, &str> {
    // stops on ^ or ~
    take_until("^")(input)
}

// fn parse_zpl_item(input: &str) -> IResult<&str, ZplFormatCommand> {
//     // Skip anything until a command starts
//     let (input, _) = nom::combinator::opt(skip_until_command).parse(input)?;

//     parse_command(input)
// }

// pub fn parse_zpl(input: &str) -> IResult<&str, Vec<ZplFormatCommand>> {
//     many0(parse_zpl_item).parse(input)
// }

fn parse_zpl_item(input: &str) -> IResult<&str, Option<ZplFormatCommand>> {
    // Skip anything until a command starts
    let (input, _) = nom::combinator::opt(skip_until_command).parse(input)?;

    // Try to parse a known command, or skip if unknown
    alt((
        map(parse_command, Some),
        map(skip_unknown_command, |_| None),
    ))
    .parse(input)
}

pub fn parse_zpl(input: &str) -> IResult<&str, Vec<ZplFormatCommand>> {
    let (input, items) = many0(parse_zpl_item).parse(input)?;
    let commands = items.into_iter().flatten().collect();
    Ok((input, commands))
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{CompressionMethod, CompressionType, GraficData, Orientation, ZplFormatCommand},
        parse::{
            parse_a, parse_fd, parse_fg, parse_fo, parse_ft, parse_ll, parse_ls, parse_pw,
            parse_zpl,
        },
    };

    #[test]
    fn parse_ll_test() {
        let input = "^LL236^LS0";
        let (remain, zpl) = parse_ll(input).unwrap();
        assert_eq!(remain, "^LS0");
        assert_eq!(zpl, ZplFormatCommand::LabelLength(236));
    }

    #[test]
    fn parse_pw_test() {
        let input = "^PW685^LL236";
        let (remain, zpl) = parse_pw(input).unwrap();
        assert_eq!(remain, "^LL236");
        assert_eq!(zpl, ZplFormatCommand::PrintWidth(685));
    }

    #[test]
    fn parse_ls_test() {
        let input = "^LS0^FT86";
        let (remain, zpl) = parse_ls(input).unwrap();
        assert_eq!(remain, "^FT86");
        assert_eq!(zpl, ZplFormatCommand::LabelShift(0));
    }

    #[test]
    fn parse_a_test() {
        let input = "^A0N,21,20^FH";
        let (remain, zpl) = parse_a(input).unwrap();
        assert_eq!(remain, "^FH");
        assert_eq!(
            zpl,
            ZplFormatCommand::Font {
                name: '0',
                orientation: Orientation::Normal,
                height: 21,
                width: 20
            }
        );
    }

    #[test]
    fn parse_fo_test() {
        let input = "^FO349,327^FT";
        let (remain, zpl) = parse_fo(input).unwrap();
        assert_eq!(remain, "^FT");
        assert_eq!(
            zpl,
            ZplFormatCommand::FieldOrigin {
                x: 349,
                y: 327,
                justification: 0
            }
        );
        let input = "^FO349,327,2^FT";
        let (remain, zpl) = parse_fo(input).unwrap();
        assert_eq!(remain, "^FT");
        assert_eq!(
            zpl,
            ZplFormatCommand::FieldOrigin {
                x: 349,
                y: 327,
                justification: 2
            }
        );
    }

    #[test]
    fn parse_ft_test() {
        let input = "^FT349,327";
        let (remain, zpl) = parse_ft(input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::FieldTypeset {
                x: 349,
                y: 327,
                justification: 0
            }
        );
        let input = "^FT349,327,2";
        let (remain, zpl) = parse_ft(input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::FieldTypeset {
                x: 349,
                y: 327,
                justification: 2
            }
        );
    }

    #[test]
    fn parse_fd_test() {
        let input = "^FDText^FS";
        let (remain, zpl) = parse_fd(input).unwrap();
        assert_eq!(remain, "^FS");
        assert_eq!(zpl, ZplFormatCommand::FieldData("Text".into()));
    }

    #[test]
    fn parse_gf_test() {
        let data = "eJytk7ENg0AMRQ8BAinFNenZBMpshdkgK1CnyAp4g2SEG4ESIYLjozr7LkqKmO7JenxsUxhVhWk1sthrVOE+fO+yGLtcwtWbOgT14TqHqDxcwmZH68BAiBr0uShMZhd2lSS6ZnbZXaCVczWbSEUVttMSohNdEeAZoowe2NEovocIQbyQ/YREN1GT76KXeIhduhxECH9DKdce51KL7LwLBQLvcuobHcAsJ3HBthPzynlefSWnuvHsc5HCrryhTG0ovUe97eRNRJfz4b5UJW8VNPrv3f/yp6VccVdm7jqXGd7xtuh/:E957";
        let input = format!("^GFA,309,988,19,:Z64:{data}^FT");
        let (remain, zpl) = parse_fg(&input).unwrap();
        assert_eq!(remain, "^FT");
        assert_eq!(
            zpl,
            ZplFormatCommand::GraficField {
                compression_type: CompressionType::Ascii,
                data_bytes: 309,
                total_bytes: 988,
                row_bytes: 19,
                data: GraficData {
                    compression_method: CompressionMethod::Zlib,
                    data: data.into()
                }
            }
        );
    }

    #[test]
    fn parse_zpl_test() {
        let input = r"^PW685
        ^LL236
        ^LS0
        ^FT86,78^A0N,51,51^FH\^CI28^FD#1001#^FS^CI27";

        let (remain, commands) = parse_zpl(input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            commands,
            vec![
                ZplFormatCommand::PrintWidth(685),
                ZplFormatCommand::LabelLength(236),
                ZplFormatCommand::LabelShift(0),
                ZplFormatCommand::FieldTypeset {
                    x: 86,
                    y: 78,
                    justification: 0
                },
                ZplFormatCommand::Font {
                    name: '0',
                    orientation: Orientation::Normal,
                    height: 51,
                    width: 51,
                },
                ZplFormatCommand::FieldData("#1001#".to_string()),
                ZplFormatCommand::FieldSeparator
            ]
        )
    }

    #[test]
    fn parse_zpl_test_2() {
        let input = std::fs::read_to_string("../zpl/zpl_real_live.txt").unwrap();
        let (remain, commands) = parse_zpl(&input).unwrap();
        assert_eq!(remain, "");
        println!("{commands:?}");
    }
}
