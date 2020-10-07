use serde_json::Value as JsonValue;

use crate::{
    parser::{ParseError, ScalarToken, Token},
    value::{ParseScalarResult},
    Value,
};

#[crate::graphql_scalar(description = "JSON serialized as a string")]
impl<S> GraphQLScalar for JsonValue
where
    S: ScalarValue,
{
    fn resolve(&self) -> Value {
        Value::scalar(self.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<JsonValue> {
        v.as_string_value()
         .and_then(|s| serde_json::from_str(s).ok())
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        if let ScalarToken::String(value) = value {
            Ok(S::from(value.to_owned()))
        } else {
            Err(ParseError::UnexpectedToken(Token::Scalar(value)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn json_from_input_value() {
        let raw = r#"{ "foo": "bar"}"#;
        let input: crate::InputValue = crate::InputValue::scalar(raw.to_string());

        let parsed: JsonValue = crate::FromInputValue::from_input_value(&input).unwrap();
        let expected: JsonValue = serde_json::from_str(raw).unwrap();

        assert_eq!(parsed, expected);
    }
}

#[cfg(test)]
mod integration_test {
    use super::*;

    use crate::{
        executor::Variables,
        schema::model::RootNode,
        types::scalars::{EmptyMutation, EmptySubscription},
        value::Value,
    };

    #[tokio::test]
    async fn test_json_serialization() {
        let example_raw: JsonValue = serde_json::from_str(
            r#"{
            "x": 2,
            "y": 42
            }
        "#,
        )
        .unwrap();
        let example_raw = example_raw.to_string();

        struct Root;

        #[crate::graphql_object]
        impl Root {
            fn example_json() -> JsonValue {
                serde_json::from_str(r#"{
                    "x": 2,
                    "y": 42
                    }
                "#).unwrap()
            }
            fn input_json(input: JsonValue) -> bool {
                input.is_array()
            }
        };

        let doc = r#"
        {
            exampleJson,
            inputJson(input: "[]"),
        }
        "#;

        let schema = RootNode::new(
            Root,
            EmptyMutation::<()>::new(), 
            EmptySubscription::<()>::new()
        );

        let (result, errs) = crate::execute(doc, None, &schema, &Variables::new(), &())
            .await
            .expect("Execution failed");

        assert_eq!(errs, []);

        assert_eq!(
            result,
            Value::object(
                vec![
                    ("exampleJson", Value::scalar(example_raw)),
                    ("inputJson", Value::scalar(true)),
                ]
                .into_iter()
                .collect()
            )
        );
    }
}
