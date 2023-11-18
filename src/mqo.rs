//! Implementation of Metasequoia object loading functions and Model class.

use std::{error::Error, io::Read, str::FromStr};

use three_d_asset::{prelude::Vector3, Indices, Positions, TriMesh};

pub struct Bone {}

pub fn load_mqo(
    is: &mut impl Read,
    pret: &mut Vec<TriMesh>,
    pname: &mut Vec<String>,
    bones: Option<&mut Vec<Bone>>,
) -> Result<(), Box<dyn Error>> {
    load_mqo_scale(is, pret, pname, bones, 1., &|| ())
}

type MqoTextureCallback = dyn Fn();

/// Load Metasequoia object with scaling and a texture callback.
pub fn load_mqo_scale(
    is: &mut impl Read,
    pret: &mut Vec<TriMesh>,
    pname: &mut Vec<String>,
    bones: Option<&mut Vec<Bone>>,
    scale: f32,
    tex_callback: &MqoTextureCallback,
) -> Result<(), Box<dyn Error>> {
    // char buf[128], *s = NULL, *name = NULL;
    // Mesh **ret = NULL;
    // Mesh *sufatr = NULL;
    // Mesh::Index atr = USHRT_MAX; /* current attribute index */
    // int num = 0;

    // if is.() || !is.good())
    // 	return 0;
    // FPOS fo(is);

    if let Some(bones) = bones {
        *bones = vec![];
    }

    /* checking signatures */
    expect_token(is, "Metasequoia")?;
    expect_token(is, "Document")?;
    expect_token(is, "Format")?;
    expect_token(is, "Text")?;
    expect_token(is, "Ver")?;
    let ver = read_token(is)?;
    if ver != "1.0" && ver != "1.1" {
        return Err("Version is not supported 1.0 or 1.1".into());
    }

    loop {
        let mut bracestack = 0;
        let Ok(s) = read_token(is) else {
            break;
        };
        match &s.to_lowercase() as &_ {
            "material" => {
                println!("reading material chunk");
                chunk_material(is)?;
                // if(!chunk_material(sufatr, &fo))
                //     return NULL;
            }
            "object" => {
                let s = read_token(is)?;
                pname.push(s);
                pret.push(chunk_object(is, scale)?);
            }
            "eof" => return Ok(()),
            _ => {
                println!("Skipping unrecognized chunk {s}");
                loop {
                    let s = read_token(is)?;
                    let ch = s.chars().next();
                    if ch == Some('{') {
                        bracestack += 1;
                    } else if ch == Some('}') {
                        bracestack -= 1;
                        if bracestack == 0 {
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_mqo() {
    let mut mqo_reader =
        std::io::BufReader::new(std::fs::File::open("A10.mqo").unwrap());
    let mut meshes = vec![];
    let mut names = vec![];
    load_mqo(&mut mqo_reader, &mut meshes, &mut names, None).unwrap();
    println!("meshes: {}", meshes.len());
}

fn chunk_material(is: &mut impl Read) -> Result<(), Box<dyn Error>> {
    // char *s;
    // int n, i, j;
    // Mesh::Attrib *patr;
    let s = read_token(is)?;
    let num_mats = s.parse::<usize>()?;
    println!("Expects {num_mats} materials");

    // ret->na = n;
    // ret->a = (Mesh::Attrib*)malloc(n * sizeof *ret->a);

    loop {
        let s = read_token(is)?;
        if s == "{" {
            break;
        }
    }

    for _i in 0..num_mats {
        // char line[512], *cur;
        // Mesh::Attrib *atr = &ret->a[i];
        // double opa = 1.;
        let line = read_line(is)?;
        let cur = line;
        let (_, s) = quotok(&cur)?;
        if s == b"}" {
            continue;
        }
        if s.is_empty() {
            return Ok(());
        }
        // ret->a[i] = atr0;
        // ret->a[i].name = (char*)malloc(strlen(s) + 1);
        // strcpy(ret->a[i].name, s);
    }

    if read_line(is)? != b"}" {
        return Err("Material chunk not closed by a brace".into());
    }

    Ok(())
}

fn chunk_object(
    is: &mut impl Read,
    scale: f32,
) -> Result<TriMesh, Box<dyn Error>> {
    // char *s, *name = NULL;
    // int n, i, j;
    // Mesh::Attrib *patr;
    // char line[512], *cur;
    // Mesh::Index atr = (Mesh::Index)-1;
    let mut shading = 0.;
    let mut facet = 0.;
    // struct Bone *bone = NULL;
    let mut mirror = false;
    let mut mirror_axis = 0;
    let mut mirrors = 0;
    // int mirrornv[3];
    let mut positions: Vec<Vector3<f32>> = vec![];
    let mut faces = vec![];
    let mut materials = vec![0u16; 0];

    let line = read_line(is)?;
    let (r, name) = quotok(&line)?;

    /* forward until vertex chunk */
    while let Ok(line) = read_line(is) {
        let (r, attr_name) = quotok(&line)?;
        if attr_name.is_empty() {
            continue;
        }
        match &attr_name.to_ascii_lowercase() as &[u8] {
            b"vertex" => {
                let (_r, num_vertices) = quotok(r)?;
                let num_vertices = parse_u8(&num_vertices)?;
                for _i in 0..num_vertices {
                    let line = read_line(is)?;
                    if line.first() == Some(&b'{') {
                        break;
                    }
                    let mut cur: &[_] = &line;
                    let mut vec = [0f32; 3];
                    for j in 0..3 {
                        let (next_cur, s) = quotok(&cur)?;
                        vec[j] =
                            std::str::from_utf8(&s)?.parse::<f32>()? * scale;
                        cur = next_cur;
                    }
                    positions.push(vec.into());
                }
                let line = read_line(is)?;
                if quotok(&line)?.1 != b"}" {
                    return Err("Vertex payload not closed by a brace".into());
                };
                // dbg!(&positions[..10]);
            }
            b"face" => {
                let (_r, num_faces) = quotok(r)?;
                let num_faces = parse_u8(&num_faces)?;
                for _i in 0..num_faces {
                    let line = read_line(is)?;
                    if line.first() == Some(&b'{') {
                        break;
                    }
                    let (mut r, s) = quotok(&line)?;
                    let dims = parse_u8(&s)?;
                    r = skip_whitespace(r);
                    if &r[..2] != b"V(" {
                        continue;
                    }
                    let mut cur = &r[2..];
                    let mut vertices = vec![0u16; dims];
                    for j in 0..dims {
                        let (next_cur, s) = quotok(&cur)?;
                        vertices[j] = parse_u8(&s)?;
                        cur = next_cur;
                    }
                    vertices.reverse();
                    faces.extend_from_slice(&vertices);
                    cur = skip_whitespace(cur);
                    if &r[..2] != b"M(" {
                        continue;
                    }
                    for _j in 0..dims {
                        let (next_cur, s) = quotok(&cur)?;
                        materials.push(parse_u8(&s)?);
                        cur = next_cur;
                    }
                }
                let line = read_line(is)?;
                if quotok(&line)?.1 != b"}" {
                    return Err("Face payload not closed by a brace".into());
                };
                // dbg!(&faces[..30]);
            }
            b"shading" => {
                let (_, s) = quotok(&r)?;
                shading = parse_u8(&s)?;
            }
            b"facet" => {
                let (_, s) = quotok(&r)?;
                facet = parse_u8(&s)?;
            }
            b"depth" => {}
            b"mirror" => {
                mirror = parse_u8::<i32>(&quotok(&r)?.1)? != 0;
            }
            b"mirror_axis" => {
                mirror_axis = parse_u8(&quotok(&r)?.1)?;

                // There could be multiple mirrors for a object, so we count it for allocating space.
                mirrors = 0;
                for m in 0..3 {
                    if (mirror_axis & (1 << m)) != 0 {
                        mirrors += 1;
                    }
                }
            }
            b"}" => break,
            _ => {
                // Unrecognized attr is not an error. Log and ignore
                println!(
                    "Unexpected attr {}",
                    String::from_utf8(attr_name)
                        .unwrap_or_else(|s| s.to_string())
                );
            }
        }
    }

    if mirror {
        for m in 0..3 {
            // Check for each axis if it's flagged for mirroring.
            if (mirror_axis & (1 << m)) != 0 {
                // Mirrored vertices have simply negated coordinate along axis perpendicular to the mirror.
                for i in 0..positions.len() {
                    let mut v = positions[i];
                    if m == 0 {
                        v.x *= -1.;
                    }
                    positions.push(v);
                }
                // mirrornv[m] = n;
                // n = ret->nv *= 2;
            }
        }
    }

    let mut ret = TriMesh {
        positions: Positions::F32(positions),
        name: String::from_utf8(name)?,
        material_name: None,
        indices: Some(Indices::U16(faces)),
        normals: None,
        tangents: None,
        uvs: None,
        colors: None,
    };
    ret.compute_normals();
    Ok(ret)
}

fn read_token(r: &mut impl Read) -> Result<String, Box<dyn Error>> {
    let mut token = vec![];
    loop {
        let mut buf = vec![0u8];
        r.read_exact(&mut buf)?;
        if buf[0].is_ascii_whitespace() {
            if token.is_empty() {
                continue;
            } else {
                break;
            }
        }
        token.push(buf[0]);
    }
    Ok(String::from_utf8(token)?)
}

fn expect_token(r: &mut impl Read, s: &str) -> Result<(), Box<dyn Error>> {
    if read_token(r)? != s {
        Err("Unexpected header".into())
    } else {
        println!("Matched {s}");
        Ok(())
    }
}

fn read_line(r: &mut impl Read) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut line_buf = vec![];
    loop {
        let mut buf = vec![0u8];
        r.read_exact(&mut buf)?;
        if buf[0] == b'\r' {
            continue;
        }
        if buf[0] == b'\n' {
            if line_buf.is_empty() {
                continue;
            } else {
                break;
            }
        }
        line_buf.push(buf[0]);
    }
    Ok(line_buf)
}

fn quotok(src: &[u8]) -> Result<(&[u8], Vec<u8>), Box<dyn Error>> {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Quote {
        None,
        DQuote,
        Paren,
    }
    let mut inquote = Quote::None;
    let mut content = vec![];
    for (i, &ch) in src.iter().enumerate() {
        if inquote != Quote::None && ch.is_ascii_whitespace() {
            continue;
            // let rest = std::str::from_utf8(&src.as_bytes()[i..])?;
            // return Ok((rest, String::from_utf8(content)?));
        }
        match inquote {
            Quote::None => {
                if ch == b'"' {
                    inquote = Quote::DQuote;
                } else if ch == b'(' {
                    inquote = Quote::Paren;
                } else if ch == b')' {
                    return Ok((&src[i..], content));
                } else if !content.is_empty() && ch.is_ascii_whitespace() {
                    return Ok((&src[i + 1..], content));
                } else if !content.is_empty() || !ch.is_ascii_whitespace() {
                    content.push(ch);
                }
                continue;
            }
            Quote::DQuote => {
                if ch == b'"' {
                    return Ok((&src[i + 1..], content));
                }
            }
            Quote::Paren => {
                if ch == b')' {
                    return Ok((&src[i + 1..], content));
                }
            }
        }
        if !content.is_empty() || !ch.is_ascii_whitespace() {
            content.push(ch);
        }
    }
    Ok((b"", content))
}

fn parse_u8<T: FromStr>(i: &[u8]) -> Result<T, Box<dyn Error>>
where
    T::Err: Error + 'static,
{
    Ok(std::str::from_utf8(i)?.parse::<T>()?)
}

fn skip_whitespace(i: &[u8]) -> &[u8] {
    let mut r = i;
    while let Some(ch) = r.first() {
        if ch.is_ascii_whitespace() {
            r = &r[1..];
        } else {
            break;
        }
    }
    r
}

#[test]
fn test_quotok() {
    let s = br#"  "hello""#;
    assert_eq!(quotok(s).unwrap(), (&b""[..], b"hello".to_vec()));
    let s = br#"  a b"#;
    assert_eq!(quotok(s).unwrap(), (&b"b"[..], b"a".to_vec()));
}