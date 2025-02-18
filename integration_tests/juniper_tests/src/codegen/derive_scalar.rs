use juniper::{
    execute, graphql_value, EmptyMutation, EmptySubscription, FromInputValue, InputValue, RootNode,
    ToInputValue, Value, Variables,
};

use crate::custom_scalar::MyScalarValue;

#[derive(Debug, PartialEq, Eq, Hash, juniper::GraphQLScalarValue)]
#[graphql(transparent, scalar = MyScalarValue)]
pub struct LargeId(i64);

#[derive(juniper::GraphQLObject)]
#[graphql(scalar = MyScalarValue)]
struct User {
    id: LargeId,
}

struct Query;

#[juniper::graphql_object(scalar = MyScalarValue)]
impl Query {
    fn user() -> User {
        User { id: LargeId(0) }
    }
}

struct Mutation;

#[juniper::graphql_object(scalar = MyScalarValue)]
impl Mutation {
    fn change_user(id: LargeId) -> User {
        User { id }
    }
}

#[test]
fn test_scalar_value_large_id() {
    let num: i64 = 4294967297;

    let input_integer: InputValue<MyScalarValue> =
        serde_json::from_value(serde_json::json!(num)).unwrap();

    let output: LargeId =
        FromInputValue::<MyScalarValue>::from_input_value(&input_integer).unwrap();
    assert_eq!(output, LargeId(num));

    let id = LargeId(num);
    let output = ToInputValue::<MyScalarValue>::to_input_value(&id);
    assert_eq!(output, InputValue::scalar(num));
}

#[tokio::test]
async fn test_scalar_value_large_query() {
    let schema = RootNode::<'_, _, _, _, MyScalarValue>::new_with_scalar_value(
        Query,
        EmptyMutation::<()>::new(),
        EmptySubscription::<()>::new(),
    );

    let doc = r#"{
        user { id }
    }"#;

    let val = Value::<MyScalarValue>::scalar(0_i64);
    assert_eq!(
        execute(doc, None, &schema, &Variables::<MyScalarValue>::new(), &()).await,
        Ok((graphql_value!({"user": {"id": val}}), vec![])),
    );
}

#[tokio::test]
async fn test_scalar_value_large_mutation() {
    let schema = RootNode::<'_, _, _, _, MyScalarValue>::new_with_scalar_value(
        Query,
        Mutation,
        EmptySubscription::<()>::new(),
    );

    let doc = r#"mutation {
        changeUser(id: 1) { id }
    }"#;

    let val = Value::<MyScalarValue>::scalar(1_i64);
    assert_eq!(
        execute(doc, None, &schema, &Variables::<MyScalarValue>::new(), &()).await,
        Ok((graphql_value!({"changeUser": {"id": val}}), vec![])),
    );

    let doc = r#"mutation {
        changeUser(id: 4294967297) { id }
    }"#;

    let val = Value::<MyScalarValue>::scalar(4294967297_i64);
    assert_eq!(
        execute(doc, None, &schema, &Variables::<MyScalarValue>::new(), &()).await,
        Ok((graphql_value!({"changeUser": {"id": val}}), vec![])),
    );
}
