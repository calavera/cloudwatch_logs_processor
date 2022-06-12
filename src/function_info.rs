/// `FunctionInfo` stores information about the function invoked
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub cloudwatch_logs_assume_role_arn: String,
}
