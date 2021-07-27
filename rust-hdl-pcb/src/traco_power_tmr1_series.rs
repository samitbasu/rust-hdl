use enum_iterator::IntoEnumIterator;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputType {
    Nominal05,
    Nominal12,
    Nominal24,
    Nominal48,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputType {
    Regulated5,
    Regulated12,
    Regulated15,
    Regulated24,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SecondaryOutputType {
    RegulatedNeg12,
    RegulatedNeg15,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Series {
    TMR1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TracoSupplyTMR {
    pub series: Series,
    pub input_type: InputType,
    pub primary_output_type: OutputType,
    pub secondary_output_type: Option<SecondaryOutputType>,
}

#[derive(Copy, Clone, Debug, IntoEnumIterator)]
pub enum OrderCode {
    TMR1_0511,
    TMR1_0512,
    TMR1_0513,
    TMR1_0515,
    TMR1_0522,
    TMR1_0523,
    TMR1_1211,
    TMR1_1212,
    TMR1_1213,
    TMR1_1215,
    TMR1_1222,
    TMR1_1223,
    TMR1_2411,
    TMR1_2412,
    TMR1_2413,
    TMR1_2415,
    TMR1_2422,
    TMR1_2423,
    TMR1_4811,
    TMR1_4812,
    TMR1_4813,
    TMR1_4815,
    TMR1_4822,
    TMR1_4823,
}

pub fn part_details(code: OrderCode) -> TracoSupplyTMR {
    let series = Series::TMR1;
    let input_type = match code {
        OrderCode::TMR1_0511
        | OrderCode::TMR1_0512
        | OrderCode::TMR1_0513
        | OrderCode::TMR1_0515
        | OrderCode::TMR1_0522
        | OrderCode::TMR1_0523 => InputType::Nominal05,
        OrderCode::TMR1_1211
        | OrderCode::TMR1_1212
        | OrderCode::TMR1_1213
        | OrderCode::TMR1_1215
        | OrderCode::TMR1_1222
        | OrderCode::TMR1_1223 => InputType::Nominal12,
        OrderCode::TMR1_2411
        | OrderCode::TMR1_2412
        | OrderCode::TMR1_2413
        | OrderCode::TMR1_2415
        | OrderCode::TMR1_2422
        | OrderCode::TMR1_2423 => InputType::Nominal24,
        OrderCode::TMR1_4811
        | OrderCode::TMR1_4812
        | OrderCode::TMR1_4813
        | OrderCode::TMR1_4815
        | OrderCode::TMR1_4822
        | OrderCode::TMR1_4823 => InputType::Nominal48,
    };
    let primary_output_type = match code {
        OrderCode::TMR1_0511
        | OrderCode::TMR1_1211
        | OrderCode::TMR1_2411
        | OrderCode::TMR1_4811 => OutputType::Regulated5,
        OrderCode::TMR1_0512
        | OrderCode::TMR1_0522
        | OrderCode::TMR1_1212
        | OrderCode::TMR1_1222
        | OrderCode::TMR1_2412
        | OrderCode::TMR1_2422
        | OrderCode::TMR1_4812
        | OrderCode::TMR1_4822 => OutputType::Regulated12,
        OrderCode::TMR1_0513
        | OrderCode::TMR1_0523
        | OrderCode::TMR1_1213
        | OrderCode::TMR1_1223
        | OrderCode::TMR1_2413
        | OrderCode::TMR1_2423
        | OrderCode::TMR1_4813
        | OrderCode::TMR1_4823 => OutputType::Regulated15,
        OrderCode::TMR1_0515
        | OrderCode::TMR1_1215
        | OrderCode::TMR1_2415
        | OrderCode::TMR1_4815 => OutputType::Regulated24,
    };
    let secondary_output_type = match code {
        OrderCode::TMR1_0522
        | OrderCode::TMR1_1222
        | OrderCode::TMR1_2422
        | OrderCode::TMR1_4822 => Some(SecondaryOutputType::RegulatedNeg12),
        OrderCode::TMR1_0523
        | OrderCode::TMR1_1223
        | OrderCode::TMR1_2423
        | OrderCode::TMR1_4823 => Some(SecondaryOutputType::RegulatedNeg15),
        _ => None,
    };
    TracoSupplyTMR {
        series,
        input_type,
        primary_output_type,
        secondary_output_type,
    }
}

#[test]
fn test_details_for_order_codes() {
    for code in OrderCode::into_enum_iter() {
        let details = part_details(code);
        let code_name = format!("{:?}", code);
        let nom_range = &code_name[5..=6];
        match nom_range {
            "05" => {
                assert_eq!(details.input_type, InputType::Nominal05)
            }
            "12" => {
                assert_eq!(details.input_type, InputType::Nominal12)
            }
            "24" => {
                assert_eq!(details.input_type, InputType::Nominal24)
            }
            "48" => {
                assert_eq!(details.input_type, InputType::Nominal48)
            }
            _ => {
                panic!("Unrecognized part code")
            }
        }
        let output_cardinality = &code_name[7..8];
        match output_cardinality {
            "1" => assert_eq!(details.secondary_output_type, None),
            "2" => assert!(details.secondary_output_type.is_some()),
            _ => {
                panic!("Unrecognized part code")
            }
        }
        let output_range = &code_name[8..];
        match output_range {
            "1" => assert_eq!(details.primary_output_type, OutputType::Regulated5),
            "2" => assert_eq!(details.primary_output_type, OutputType::Regulated12),
            "3" => assert_eq!(details.primary_output_type, OutputType::Regulated15),
            "5" => assert_eq!(details.primary_output_type, OutputType::Regulated24),
            _ => {
                panic!("Unrecognized part code")
            }
        }
        if let Some(secondary) = details.secondary_output_type {
            match secondary {
                SecondaryOutputType::RegulatedNeg12 => {
                    assert_eq!(details.primary_output_type, OutputType::Regulated12)
                }
                SecondaryOutputType::RegulatedNeg15 => {
                    assert_eq!(details.primary_output_type, OutputType::Regulated15)
                }
            }
        }
    }
}
