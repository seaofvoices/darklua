use std::path::{Component, Path, PathBuf};

use crate::nodes::{
    Expression, FieldExpression, FunctionCall, Identifier, Prefix, StringExpression,
};

use super::resolver::Resolver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobloxIndexingStrategy {
    FindFirstChild,
    WaitForChild,
}

impl RobloxIndexingStrategy {
    pub fn apply(&self, value: Prefix, instance_name: &str) -> Prefix {
        match self {
            RobloxIndexingStrategy::FindFirstChild => FunctionCall::from_prefix(value)
                .with_method("FindFirstChild")
                .with_argument(StringExpression::from_value(instance_name))
                .into(),
            RobloxIndexingStrategy::WaitForChild => FunctionCall::from_prefix(value)
                .with_method("WaitForChild")
                .with_argument(StringExpression::from_value(instance_name))
                .into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobloxRootLocation {
    Service(String),
    Script,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobloxPath {
    Parent,
    Child(String),
}

impl RobloxPath {
    pub fn child<IntoString: Into<String>>(name: IntoString) -> Self {
        Self::Child(name.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobloxLocation {
    root: RobloxRootLocation,
    path: Vec<RobloxPath>,
}

impl RobloxLocation {
    pub fn push(&mut self, path: RobloxPath) {
        self.path.push(path);
    }

    pub fn with_path(mut self, path: RobloxPath) -> Self {
        self.path.push(path);
        self
    }

    pub fn into_expression(&self, strategy: &RobloxIndexingStrategy) -> Expression {
        let mut value: Prefix = match self.root {
            RobloxRootLocation::Service(_) => todo!(),
            RobloxRootLocation::Script => Identifier::new("script").into(),
        };

        for path in self.path.iter() {
            value = match path {
                RobloxPath::Parent => FieldExpression::new(value, "Parent").into(),
                RobloxPath::Child(name) => strategy.apply(value, name),
            }
        }

        value.into()
    }
}

impl From<RobloxRootLocation> for RobloxLocation {
    fn from(root: RobloxRootLocation) -> Self {
        Self {
            root,
            path: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RobloxLibraryResolver {
    indexing_strategy: RobloxIndexingStrategy,
    library_root: PathBuf,
    file_location: PathBuf,
    dependencies: RobloxLocation,
}

impl RobloxLibraryResolver {
    pub fn new(
        indexing_strategy: RobloxIndexingStrategy,
        library_root: PathBuf,
        file_location: PathBuf,
        dependencies: RobloxLocation,
    ) -> Self {
        Self {
            indexing_strategy,
            library_root,
            file_location,
            dependencies,
        }
    }

    fn get_location(&self, path: &Path) -> Result<RobloxLocation, ()> {
        let mut components = path.components();

        let mut location = match components.next().ok_or_else(|| todo!())? {
            // this one happens only on windows and should probably be invalid
            Component::Prefix(_) => todo!(),
            Component::RootDir => todo!(),
            Component::CurDir => {
                let mut location = RobloxLocation::from(RobloxRootLocation::Script);
                // files that turn the directory into any kind of script instance do not
                // need an initial `.Parent`. Their siblings in the file hierarchy will
                // become their children in the Roblox instance tree
                if !matches!(
                    self.file_location
                        .file_name()
                        .ok_or_else(|| todo!())?
                        .to_str()
                        .expect("valid utf-8"),
                    "init.lua" | "init.server.lua" | "init.client.lua"
                ) {
                    location.push(RobloxPath::Parent);
                }
                location
            }
            Component::ParentDir => todo!(),
            Component::Normal(_) => todo!(),
        };

        for component in components {
            match component {
                // this one happens only on windows and should probably be invalid
                Component::Prefix(_) => todo!(),
                Component::RootDir => todo!(),
                Component::CurDir => todo!(),
                Component::ParentDir => {
                    location.push(RobloxPath::Parent);
                }
                Component::Normal(name) => {
                    // TODO report utf8 error
                    let child = name.to_str().expect("valid utf-8").to_owned();
                    location.push(RobloxPath::Child(child));
                }
            }
        }

        Ok(location)
    }
}

impl Resolver for RobloxLibraryResolver {
    fn resolve(&self, path: &Path) -> Option<Expression> {
        let location = self.get_location(path).ok()?;

        Some(
            FunctionCall::from_name("require")
                .with_argument(location.into_expression(&self.indexing_strategy))
                .into(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nodes::LastStatement;
    use crate::Parser;

    macro_rules! test_location {
        ($strategy:ident => $($name:ident($location:expr) => $input:literal),* $(,)?) => {
            test_location!(
                $(
                    $name($strategy, $location) => $input,
                )*
            );
        };
        ($($name:ident($strategy:ident, $location:expr) => $input:literal),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let block_input = format!("return {}", $input);
                    let block = Parser::default().parse(&block_input)
                        .expect(&format!("failed to parse `{}`", $input));

                    let return_statement = match block.get_last_statement().unwrap() {
                        LastStatement::Return(return_statement) => return_statement,
                        _ => unreachable!(),
                    };
                    assert_eq!(return_statement.len(), 1);
                    let expect_expression = return_statement.iter_expressions().next().unwrap();

                    let location: RobloxLocation = $location.into();
                    let expression = location.into_expression(&RobloxIndexingStrategy::$strategy);

                    pretty_assertions::assert_eq!(&expression, expect_expression);
                }
            )*
        };
    }

    mod find_first_child {
        use super::*;

        test_location!(FindFirstChild =>
            script(RobloxRootLocation::Script) => "script",
            script_parent(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::Parent)
            ) => "script.Parent",
            script_child(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::child("InstanceName"))
            ) => "script:FindFirstChild('InstanceName')",
            script_parent_child(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::Parent)
                    .with_path(RobloxPath::child("InstanceName"))
            ) => "script.Parent:FindFirstChild('InstanceName')",
            script_nested_child(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::Parent)
                    .with_path(RobloxPath::child("A"))
                    .with_path(RobloxPath::child("B"))
                    .with_path(RobloxPath::child("C"))
            ) => "script.Parent:FindFirstChild('A'):FindFirstChild('B'):FindFirstChild('C')",
        );
    }

    mod wait_for_child {
        use super::*;

        test_location!(WaitForChild =>
            script(RobloxRootLocation::Script) => "script",
            script_parent(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::Parent)
            ) => "script.Parent",
            script_child(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::child("InstanceName"))
            ) => "script:WaitForChild('InstanceName')",
            script_parent_child(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::Parent)
                    .with_path(RobloxPath::child("InstanceName"))
            ) => "script.Parent:WaitForChild('InstanceName')",
            script_nested_child(
                RobloxLocation::from(RobloxRootLocation::Script)
                    .with_path(RobloxPath::Parent)
                    .with_path(RobloxPath::child("A"))
                    .with_path(RobloxPath::child("B"))
                    .with_path(RobloxPath::child("C"))
            ) => "script.Parent:WaitForChild('A'):WaitForChild('B'):WaitForChild('C')",
        );
    }
}
