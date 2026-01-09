use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{tag, take_till, take_until},
        take,
    },
    character::complete::{
        alpha1, alphanumeric1, anychar, char, i8 as parse_i8, isize as parse_isize, line_ending,
        multispace0, u8 as parse_u8, usize as parse_usize,
    },
    combinator::{complete, cut, map, not, opt, peek},
    error::{Error, ErrorKind},
    multi::{many_till, many1},
    number::complete::float as parse_float,
    sequence::{preceded, tuple},
};

use crate::{
    BarcodeType, Code128Mode, Color, ParseError, ParseErrorKind, TextBlockJustification,
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

fn parse_fx(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("^FX")(input)?;
    let (input, _) = take_till(|c| c == '\n' || c == '\r')(input)?;
    let (input, _) = opt(line_ending).parse(input)?;
    Ok((input, ()))
}

fn parse_mm(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("^MM")(input)?;
    let (input, (_, _, _)) = tuple((alpha1, opt(char(',')), opt(alpha1)))(input)?;
    Ok((input, ()))
}

fn parse_md(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("^MD")(input)?;
    let (input, _) = parse_i8(input)?;
    Ok((input, ()))
}

fn parse_fh(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^FH")(input)?;
    let (input, ch) = anychar(input)?;
    Ok((input, ZplFormatCommand::FieldHexIndicator { char: ch }))
}

fn parse_ci(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^CI")(input)?;
    let (input, num) = parse_u8(input)?;
    let mapping_parser = complete(tuple((char(','), parse_u8, char(','), parse_u8)));
    // let (input, mapping) = many0(parse_mapping_strict).parse(input)?;
    let (input, (mapping, _)) = many_till(mapping_parser, peek(not(char(',')))).parse(input)?;
    let mapping = mapping.into_iter().map(|(_, x, _, y)| (x, y)).collect();
    Ok((input, ZplFormatCommand::CharacterSet { num, mapping }))
}

fn parse_fb(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = tag("^FB")(input)?;
    let (input, width) = opt(parse_usize).parse(input)?;
    let (input, lines) = opt((char(','), parse_usize)).parse(input)?;
    let (input, line_spacing) = opt((char(','), parse_isize)).parse(input)?;
    let (input, justification) = opt((char(','), alpha1)).parse(input)?;
    let (input, hanging_indent) = opt((char(','), parse_usize)).parse(input)?;

    let width = width.unwrap_or(0);
    let lines = lines.map(|(_, l)| l).unwrap_or(0);
    let line_spacing = line_spacing.map(|(_, l)| l).unwrap_or(0);
    let hanging_indent = hanging_indent.map(|(_, h)| h).unwrap_or(0);

    let justification = match justification.map(|(_, j)| j) {
        Some(j) if j == "L" => TextBlockJustification::Left,
        Some(j) if j == "R" => TextBlockJustification::Right,
        Some(j) if j == "C" => TextBlockJustification::Center,
        Some(j) if j == "J" => TextBlockJustification::Justified,
        Some(j) => TextBlockJustification::Left,
        None => TextBlockJustification::Left,
    };

    Ok((
        input,
        ZplFormatCommand::FieldBlock {
            width,
            lines: lines,
            line_spacing,
            justification,
            hanging_indent,
        },
    ))
}

fn parse_pq(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("^PQ")(input)?;
    let (input, _) = opt(parse_usize).parse(input)?;
    let (input, _) = opt((char(','), parse_usize)).parse(input)?;
    let (input, _) = opt((char(','), parse_usize)).parse(input)?;
    let (input, _) = opt((char(','), alpha1)).parse(input)?;
    let (input, _) = opt((char(','), alpha1)).parse(input)?;
    Ok((input, ()))
}

/// parse ^XA as start of label definition
fn parse_xa(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("^XA")(input)?;
    Ok((input, ()))
}

/// parse ^XZ as end of label definition
fn parse_xz(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("^XZ")(input)?;
    Ok((input, ()))
}

pub fn parse_command(input: &str) -> IResult<&str, ZplFormatCommand> {
    alt((
        parse_fo, parse_fd, parse_a, parse_fg, parse_ft, parse_ll, parse_ls, parse_pw, parse_fs,
        parse_cf, parse_gb, parse_fr, parse_by, parse_bc, parse_be, parse_ci, parse_fh,
        parse_fb, // add more commands here
    ))
    .parse(input)
}

/// Parse a single ZPL item (command with optional whitespace)
fn parse_zpl_item(input: &str) -> IResult<&str, ZplFormatCommand> {
    let (input, _) = multispace0(input)?; // Skip whitespace only
    let (input, _) = opt(parse_fx).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(parse_md).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(parse_mm).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(parse_pq).parse(input)?;
    let (input, _) = multispace0(input)?;

    // STOP on ^XZ (terminator)
    if peek(parse_xz).parse(input).is_ok() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Eof,
        )));
    }

    cut(parse_command).parse(input)
}

/// Internal parser - returns IResult
fn parse_zpl_intern(input: &str) -> IResult<&str, Vec<ZplFormatCommand>> {
    let (input, commands) = many1(parse_zpl_item).parse(input)?;
    Ok((input, commands))
}

// extract all parts starting with ^XA and ending with ^XZ
fn find_labels(input: &str) -> Vec<Result<&str, ParseError>> {
    let mut labels = Vec::new();
    let mut rest = input;

    while let Some(xa_pos) = rest.find("^XA") {
        let after_xa = &rest[xa_pos + 3..];
        if let Some(xz_pos) = after_xa.find("^XZ") {
            let end = xa_pos + 3 + xz_pos + 3;
            labels.push(Ok(&rest[xa_pos..end]));
            rest = &rest[end..];
        } else {
            labels.push(Err(ParseError {
                kind: ParseErrorKind::MissingCommand,
                message: "^XZ".to_string(),
            }));
            break;
        }
    }

    labels
}

pub fn parse_zpl(input: &str) -> Result<Vec<ZplFormatCommand>, ParseError> {
    // extract labels
    let labels = find_labels(input);
    let input = labels.last().ok_or(ParseError {
        kind: ParseErrorKind::MissingCommand,
        message: "^XA".to_string(),
    })?;

    let input = input.as_deref().map_err(|err| err.clone())?;

    // strip ^XA
    let (input, _) = parse_xa(input)?;

    // parse content
    let (_, commands) = parse_zpl_intern(input)
        .map_err(|err| <nom::Err<nom::error::Error<&str>> as Into<ParseError>>::into(err))?;

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        BarcodeType, Code128Mode, Color, Justification, ParseError, ParseErrorKind,
        TextBlockJustification,
        commands::{CompressionMethod, CompressionType, GraficData, Orientation, ZplFormatCommand},
        parse::{
            parse_a, parse_bc, parse_be, parse_by, parse_cf, parse_ci, parse_fb, parse_fd,
            parse_fg, parse_fh, parse_fo, parse_fr, parse_ft, parse_fx, parse_gb, parse_ll,
            parse_ls, parse_md, parse_mm, parse_pq, parse_pw, parse_zpl, parse_zpl_intern,
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
    fn parse_fx_test() {
        let input = "^FX this is a comment and even a ^FO may appear here\r\n^FT";
        let (remain, zpl) = parse_fx(input).unwrap();
        assert_eq!(remain, "^FT")
    }

    #[test]
    fn parse_mm_test() {
        let input = "^MMT";
        let (remain, zpl) = parse_mm(&input).unwrap();
        assert_eq!(remain, "");

        let input = "^MMT,Y";
        let (remain, zpl) = parse_mm(&input).unwrap();
        assert_eq!(remain, "")
    }

    #[test]
    fn parse_md_test() {
        let input = "^MD-30";
        let (remain, zpl) = parse_md(&input).unwrap();
        assert_eq!(remain, "");
    }

    #[test]
    fn parse_fh_test() {
        let input = "^FH\\";
        let (remain, zpl) = parse_fh(&input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(zpl, ZplFormatCommand::FieldHexIndicator { char: '\\' })
    }

    #[test]
    fn parse_ci_test() {
        let input = "^CI28";
        let (remain, zpl) = parse_ci(&input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::CharacterSet {
                num: 28,
                mapping: HashMap::new()
            }
        );

        let input = "^CI0,36,21";
        let (remain, zpl) = parse_ci(&input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::CharacterSet {
                num: 0,
                mapping: [(36, 21)].into()
            }
        )
    }

    #[test]
    fn should_error_on_parse_ci_test() {
        let input = "^CI0,1";
        let err = parse_ci(&input).unwrap_err();
        assert_eq!(
            err,
            nom::Err::Error(nom::error::Error {
                input: "",
                code: nom::error::ErrorKind::Char
            })
        )
    }

    #[test]
    fn parse_fb_test() {
        let input = "^FB500,5";
        let (remain, zpl) = parse_fb(&input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::FieldBlock {
                width: 500,
                lines: 5,
                line_spacing: 0,
                justification: TextBlockJustification::Left,
                hanging_indent: 0
            }
        );

        let input = "^FB500,5,1,R,1";
        let (remain, zpl) = parse_fb(&input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(
            zpl,
            ZplFormatCommand::FieldBlock {
                width: 500,
                lines: 5,
                line_spacing: 1,
                justification: TextBlockJustification::Right,
                hanging_indent: 1
            }
        );
    }

    #[test]
    fn parse_pq_test() {
        let input = "^PQ10";
        let (remain, _) = parse_pq(&input).unwrap();
        assert_eq!(remain, "");

        let input = "^PQ10,0,0,Y,N";
        let (remain, _) = parse_pq(&input).unwrap();
        assert_eq!(remain, "");
    }

    #[test]
    fn parse_zpl_intern_test() {
        let input = r"^PW685
        ^LL236
        ^LS0
        ^FT86,78^A0N,51,51^FD#1001#^FS^XZ";

        let (remain, commands) = parse_zpl_intern(input).unwrap();

        assert_eq!(remain, "^XZ");
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
    fn should_error_on_missing_xa() {
        let input = "^FDTest^FS";
        let err = parse_zpl(input).unwrap_err();
        assert_eq!(
            err,
            ParseError {
                kind: ParseErrorKind::MissingCommand,
                message: "^XA".to_string()
            }
        )
    }

    #[test]
    fn should_error_on_missing_xz() {
        let input = "^XA^FDTest^FS";
        let err = parse_zpl(input).unwrap_err();
        assert_eq!(
            err,
            ParseError {
                kind: ParseErrorKind::MissingCommand,
                message: "^XZ".to_string()
            }
        )
    }

    #[test]
    fn should_error_on_invalid_syntax_command() {
        let input = "^XAInvalidCommand^XZ";
        let err = dbg!(parse_zpl(input)).unwrap_err();
        assert_eq!(
            err,
            ParseError {
                kind: ParseErrorKind::InvalidSyntax,
                message: "InvalidCom".to_string()
            }
        )
    }

    #[test]
    fn should_error_on_unknown_command() {
        let input = "^XA^FT20,20^Unknown^CF0,60^XZ";
        let err = parse_zpl(input).unwrap_err();
        assert_eq!(
            err,
            ParseError {
                kind: ParseErrorKind::InvalidSyntax,
                message: "^Unknown^C".to_string()
            }
        );
    }

    #[test]
    fn parse_zpl_test_2() {
        let input = std::fs::read_to_string("../zpl/examples/zpl_real_live.txt").unwrap();
        let commands = parse_zpl(&input).unwrap();
        println!("{commands:?}");
    }

    #[test]
    fn parse_zpl_test_with_host_commands() {
        let input = std::fs::read_to_string("../zpl/examples/render_with_images.txt").unwrap();
        let commands = parse_zpl(&input).unwrap();
        println!("{commands:?}");
    }
}
