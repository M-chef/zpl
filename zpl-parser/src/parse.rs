use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{tag, take_until, take_while1},
        take,
    },
    character::complete::{alpha1, alphanumeric1, char, u8 as parse_u8, usize as parse_usize},
    combinator::{map, opt, recognize},
    error::{Error, ErrorKind},
    multi::many0,
    number::complete::float as parse_float,
    sequence::{preceded, tuple},
};

use crate::{
    BarcodeType, Code128Mode, Color,
    commands::{CompressionMethod, CompressionType, GraficData, Orientation, ZplFormatCommand},
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
    let (input, length) = parse_usize(input)?;
    Ok((input, ZplFormatCommand::LabelShift(length)))
}

fn parse_cf(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^CF")(input)?;
    let (input, (name, _, height, _, width)) = tuple((
        take(1u8),
        char(','),
        opt(parse_usize),
        opt(char(',')),
        opt(parse_usize),
    ))(input)?;

    let (height, width) = match (height, width) {
        (None, None) => return IResult::Err(nom::Err::Error(Error::new(input, ErrorKind::NoneOf))),
        (None, Some(w)) => (w, w),
        (Some(h), None) => (h, h),
        (Some(h), Some(w)) => (h, w),
    };

    let name = name.chars().next().unwrap_or('A');
    Ok((
        input,
        ZplFormatCommand::ChangeFont {
            name,
            height,
            width,
        },
    ))
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

    let font = font.chars().next().unwrap_or('A');
    let (_, orientation) = Orientation::try_from_str(orientation)?;
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

fn parse_coordinates(input: &str) -> IResult<&str, (usize, usize, Option<u8>)> {
    alt((
        map(
            tuple((parse_usize, char(','), parse_usize, char(','), parse_u8)),
            |(x, _, y, _, z)| (x, y, Some(z)),
        ),
        map(tuple((parse_usize, char(','), parse_usize)), |(x, _, y)| {
            (x, y, None)
        }),
    ))
    .parse(input)
}

pub fn parse_fo(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, (x, y, justification)) = preceded(tag("^FO"), parse_coordinates).parse(input)?;
    Ok((
        input,
        ZplFormatCommand::FieldOrigin {
            x,
            y,
            justification: justification.into(),
        },
    ))
}

pub fn parse_ft(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, (x, y, justification)) = preceded(tag("^FT"), parse_coordinates).parse(input)?;
    Ok((
        input,
        ZplFormatCommand::FieldTypeset {
            x,
            y,
            justification: justification.into(),
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
    let (_, img_data) = take_until(":")(img_data)?;
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
        ZplFormatCommand::GraphicField {
            compression_type,
            data_bytes,
            total_bytes,
            row_bytes,
            data,
        },
    ))
}

fn parse_gb(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^GB")(input)?;
    let (input, graphical_box) = take_until("^FS")(input)?;
    let (_, (width, _, height, _, thickness, _, color, _, rounding)) = tuple((
        opt(parse_usize),
        char(','),
        opt(parse_usize),
        char(','),
        opt(parse_usize),
        opt(char(',')),
        opt(alpha1),
        opt(char(',')),
        opt(parse_u8),
    ))(graphical_box)?;

    let thickness = thickness.unwrap_or(1);
    let width = width.unwrap_or(thickness);
    let height = height.unwrap_or(thickness);
    let color: Color = color.into();
    let rounding = rounding.unwrap_or(0);

    Ok((
        input,
        ZplFormatCommand::GraphicalBox {
            width,
            height,
            thickness,
            color,
            rounding,
        },
    ))
}

fn parse_fr(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^FR")(input)?;
    Ok((input, ZplFormatCommand::Inverted))
}

fn parse_by(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^BY")(input)?;
    let (input, (width, _, width_ratio, _, height)) = tuple((
        opt(parse_u8),
        opt(char(',')),
        opt(parse_float),
        opt(char(',')),
        opt(parse_usize),
    ))(input)?;

    let width = width.unwrap_or(2);
    let width_ratio = width_ratio.unwrap_or(3.);
    let height = height.unwrap_or(10);

    Ok((
        input,
        ZplFormatCommand::BarcodeConfig {
            width,
            width_ratio,
            height,
        },
    ))
}

// fn parse_b3(input: &str) -> IResult<&str, ZplFormatCommand> {
//     let (input, _) = tag("^B3")(input)?;
//     // let
//     // Ok((input, ZplFormatCommand::))
// }

fn parse_b7(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^B7")(input)?;
    Ok((input, ZplFormatCommand::Inverted))
}

fn parse_b8(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^B8")(input)?;
    Ok((input, ZplFormatCommand::Inverted))
}

fn parse_bc(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^BC")(input)?;

    let (input, rest) = take_until("^FD")(input)?;

    let (input, (orientation, _, height, _, line, _, line_above, _, check_digit, _, mode)) =
        match rest.is_empty() {
            true => (
                input,
                (
                    None, None, None, None, None, None, None, None, None, None, None,
                ),
            ),
            false => {
                let (_, params) = tuple((
                    opt(take(1usize)),
                    opt(char(',')),
                    opt(parse_usize),
                    opt(char(',')),
                    opt(alpha1),
                    opt(char(',')),
                    opt(alpha1),
                    opt(char(',')),
                    opt(alpha1),
                    opt(char(',')),
                    opt(alpha1),
                ))(rest)?;
                (input, params)
            }
        };

    let orientation = match orientation {
        Some(o) => {
            let result = Orientation::try_from_str(o);
            result
                .map(|(_, orientation)| orientation)
                .unwrap_or(Orientation::Normal)
        }
        None => Orientation::Normal,
    };

    let show_text = line
        .map(|line| match line {
            "N" => false,
            _ => true,
        })
        .unwrap_or(true);

    let text_above = line_above
        .map(|l_above| match l_above {
            "N" => false,
            _ => true,
        })
        .unwrap_or(true);

    let check_digit = check_digit
        .map(|digit| match digit {
            "N" => false,
            _ => true,
        })
        .unwrap_or(true);

    let mode = mode
        .map(|mode| match mode {
            "N" => Code128Mode::Normal,
            "U" => Code128Mode::Ucc,
            "D" => Code128Mode::Ean,
            "A" => Code128Mode::Auto,
            _ => Code128Mode::Normal,
        })
        .unwrap_or(Code128Mode::Normal);

    Ok((
        input,
        ZplFormatCommand::Barcode(BarcodeType::Code128 {
            orientation,
            height,
            show_text,
            text_above,
            check_digit,
            mode,
        }),
    ))
}

fn parse_be(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^BE")(input)?;
    let (input, rest) = take_until("^FD")(input)?;

    let (input, (orientation, _, height, _, line, _, line_above)) = match rest.is_empty() {
        true => (input, (None, None, None, None, None, None, None)),
        false => {
            let (_, params) = tuple((
                opt(take(1usize)),
                opt(char(',')),
                opt(parse_usize),
                opt(char(',')),
                opt(alpha1),
                opt(char(',')),
                opt(alpha1),
            ))(rest)?;
            (input, params)
        }
    };

    let orientation = match orientation {
        Some(o) => {
            let result = Orientation::try_from_str(o);
            result
                .map(|(_, orientation)| orientation)
                .unwrap_or(Orientation::Normal)
        }
        None => Orientation::Normal,
    };

    let show_text = line
        .map(|line| match line {
            "N" => false,
            _ => true,
        })
        .unwrap_or(true);

    let text_above = line_above
        .map(|l_above| match l_above {
            "N" => false,
            _ => true,
        })
        .unwrap_or(true);

    Ok((
        input,
        ZplFormatCommand::Barcode(BarcodeType::Ean13 {
            orientation,
            height,
            show_text,
            text_above,
        }),
    ))
}

fn parse_bq(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^BQ")(input)?;
    Ok((input, ZplFormatCommand::Inverted))
}

fn parse_bx(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^BX")(input)?;
    Ok((input, ZplFormatCommand::Inverted))
}

fn parse_command_name(input: &str) -> IResult<&str, &str> {
    // Matches e.g. "^FO", "^FD", "~DG", "^XYZ"
    recognize(preceded(
        nom::character::complete::one_of("^~"),
        take_while1(|c: char| c.is_ascii_uppercase()),
    ))
    .parse(input)
}

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
        map(parse_cf, |c| c),
        map(parse_gb, |c| c),
        map(parse_fr, |c| c),
        map(parse_by, |c| c),
        map(parse_bc, |c| c),
        map(parse_be, |c| c),
        // add more commands here
    ))
    .parse(input)
}

pub fn skip_until_command(input: &str) -> IResult<&str, &str> {
    // stops on ^ or ~
    take_until("^")(input)
}

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
        BarcodeType, Code128Mode, Color, Justification,
        commands::{CompressionMethod, CompressionType, GraficData, Orientation, ZplFormatCommand},
        parse::{
            parse_a, parse_bc, parse_be, parse_by, parse_cf, parse_fd, parse_fg, parse_fo,
            parse_fr, parse_ft, parse_gb, parse_ll, parse_ls, parse_pw, parse_zpl,
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
    fn parse_cf_test() {
        let input = "^CF0,60";
        let (remain, zpl) = parse_cf(input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::ChangeFont {
                name: '0',
                height: 60,
                width: 60,
            }
        );

        let input = "^CF0,60,30";
        let (remain, zpl) = parse_cf(input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::ChangeFont {
                name: '0',
                height: 60,
                width: 30,
            }
        );
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
                justification: Justification::Left
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
                justification: Justification::Auto
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
                justification: Justification::Left
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
                justification: Justification::Auto
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
        let data = "eJytk7ENg0AMRQ8BAinFNenZBMpshdkgK1CnyAp4g2SEG4ESIYLjozr7LkqKmO7JenxsUxhVhWk1sthrVOE+fO+yGLtcwtWbOgT14TqHqDxcwmZH68BAiBr0uShMZhd2lSS6ZnbZXaCVczWbSEUVttMSohNdEeAZoowe2NEovocIQbyQ/YREN1GT76KXeIhduhxECH9DKdce51KL7LwLBQLvcuobHcAsJ3HBthPzynlefSWnuvHsc5HCrryhTG0ovUe97eRNRJfz4b5UJW8VNPrv3f/yp6VccVdm7jqXGd7xtuh/";
        let checksum = ":E957";
        let input = format!("^GFA,309,988,19,:Z64:{data}{checksum}^FT");
        let (remain, zpl) = parse_fg(&input).unwrap();
        assert_eq!(remain, "^FT");
        assert_eq!(
            zpl,
            ZplFormatCommand::GraphicField {
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
    fn parse_gb_test() {
        let input = format!("^GB100,100,100^FS");
        let (remain, zpl) = parse_gb(&input).unwrap();
        assert_eq!(remain, "^FS");
        assert_eq!(
            zpl,
            ZplFormatCommand::GraphicalBox {
                width: 100,
                height: 100,
                thickness: 100,
                color: Color::Black,
                rounding: 0,
            }
        );
    }

    #[test]
    fn parse_fr_test() {
        let input = format!("^FR^FDTest^FS");
        let (remain, zpl) = parse_fr(&input).unwrap();
        assert_eq!(remain, "^FDTest^FS");
        assert_eq!(zpl, ZplFormatCommand::Inverted);
    }

    #[test]
    fn parse_by_test() {
        let input = format!("^BY5,2,270^FO100,550");
        let (remain, zpl) = parse_by(&input).unwrap();
        assert_eq!(remain, "^FO100,550");
        assert_eq!(
            zpl,
            ZplFormatCommand::BarcodeConfig {
                width: 5,
                width_ratio: 2.,
                height: 270
            }
        );
    }

    #[test]
    fn parse_bc_test() {
        let input = format!("^BCN,50,Y,N,N,A^FD12345678^FS");
        let (remain, zpl) = parse_bc(&input).unwrap();
        assert_eq!(remain, "^FD12345678^FS");
        assert_eq!(
            zpl,
            ZplFormatCommand::Barcode(BarcodeType::Code128 {
                orientation: Orientation::Normal,
                height: Some(50),
                show_text: true,
                text_above: false,
                check_digit: false,
                mode: Code128Mode::Auto
            })
        );
    }

    #[test]
    fn parse_bc_blank_test() {
        let input = format!("^BC^FD12345678^FS");
        let (remain, zpl) = parse_bc(&input).unwrap();
        assert_eq!(remain, "^FD12345678^FS");
        assert_eq!(
            zpl,
            ZplFormatCommand::Barcode(BarcodeType::Code128 {
                orientation: Orientation::Normal,
                height: None,
                show_text: true,
                text_above: true,
                check_digit: true,
                mode: Code128Mode::Normal
            })
        );
    }

    #[test]
    fn parse_be_test() {
        let input = format!("^BEN,50,Y,N^FD12345678^FS");
        let (remain, zpl) = parse_be(&input).unwrap();
        assert_eq!(remain, "^FD12345678^FS");
        assert_eq!(
            zpl,
            ZplFormatCommand::Barcode(BarcodeType::Ean13 {
                orientation: Orientation::Normal,
                height: Some(50),
                show_text: true,
                text_above: false,
            })
        );
    }

    #[test]
    fn parse_be_blank_test() {
        let input = format!("^BE^FD12345678^FS");
        let (remain, zpl) = parse_be(&input).unwrap();
        assert_eq!(remain, "^FD12345678^FS");
        assert_eq!(
            zpl,
            ZplFormatCommand::Barcode(BarcodeType::Ean13 {
                orientation: Orientation::Normal,
                height: None,
                show_text: true,
                text_above: true,
            })
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
                    justification: Justification::Left
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
        let input = std::fs::read_to_string("../zpl/examples/zpl_real_live.txt").unwrap();
        let (remain, commands) = parse_zpl(&input).unwrap();
        assert_eq!(remain, "");
        println!("{commands:?}");
    }
}
