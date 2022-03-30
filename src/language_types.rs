pub mod annotations{
    pub struct DefineAnnotation {
        pub key: String,
        pub value: String
    }
    pub struct VersionAnnotation{
        pub major_version: u8,
        pub minor_version: u8,
    }

    pub struct IncludeAnnotation{
        pub path: String,
        pub content: String
    }

    pub struct ModuleAnnotation{

    }
}

pub struct GlobalOption {
    name: String
}

pub mod objects{

    #[derive(PartialEq, Eq)]
pub enum ObjectKind{
        Source,
        Destination,
        Log,
        Filter,
        Parser,
        RewriteRule,
        Template
    }

    pub struct Parameter{

    }

    pub trait Object {
        fn get_id(&self) -> &str;
        fn get_optional_parameters(&self) -> Vec<Parameter>;
        fn get_mandatory_parameters(&self) -> Vec<Parameter>;
        fn get_kind(&self) -> ObjectKind;
    }


    // Abstract Class(Object)
    // doSomething()

    // Source : Object
    // override doSomething()

    // Destination : Object
    // override doSomething()

    
    // Rewrite : Object

}