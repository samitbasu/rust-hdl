use num_bigint::{BigInt, Sign};
use svg::node::element::path::Data;
use svg::node::element::Path;
use vcd::Value;

pub fn rect(x0: u32, y0: u32, x1: u32, y1: u32, color: &str) -> Path {
    let data = Data::new()
        .move_to((x0, y0))
        .line_to((x1, y0))
        .line_to((x1, y1))
        .line_to((x0, y1))
        .close();
    let path = Path::new()
        .set("fill", color)
        .set("stroke", "none")
        .set("stroke-width", 0)
        .set("d", data);
    path
}

pub fn line(x0: u32, y0: u32, x1: u32, y1: u32, color: &str) -> Path {
    let data = Data::new().move_to((x0, y0)).line_to((x1, y1));
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", color)
        .set("stroke-width", 1)
        .set("d", data);
    path
}

pub fn value_to_bigint(v: &[Value]) -> Result<BigInt, anyhow::Error> {
    let bits = v
        .iter()
        .map(|x| match value_to_bool(x) {
            Ok(b) => {
                if b {
                    Ok(1_u8)
                } else {
                    Ok(0_u8)
                }
            }
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BigInt::from_radix_le(Sign::Plus, &bits, 2).unwrap())
}

pub fn value_to_bool(v: &Value) -> Result<bool, anyhow::Error> {
    return match v {
        Value::V0 => Ok(false),
        Value::V1 => Ok(true),
        _ => Err(anyhow::Error::msg("Unsupported scalar signal type!")),
    };
}

pub fn time_label(val: u64) -> String {
    if val < 1000 {
        format!("{}ps", val)
    } else if val < 1_000_000 {
        format!("{}ns", val / 1_000)
    } else if val < 1_000_000_000 {
        format!("{}us", val / 1_000_000)
    } else {
        format!("{}ms", val / 1_000_000_000)
    }
}
