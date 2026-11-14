use crate::types::core::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Int,
    BigInt,
    SmallInt,
    Varchar(u32),
    Text,
    Boolean,
    Timestamp,
    Date,
    Float,
    Double,
    Blob,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
    pub primary_key: bool,
    pub unique: bool,
    pub auto_increment: bool,
}

impl ColumnDef {
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: false,
            default: None,
            primary_key: false,
            unique: false,
            auto_increment: false,
        }
    }

    #[must_use]
    pub const fn nullable(mut self, v: bool) -> Self {
        self.nullable = v;
        self
    }

    #[must_use]
    pub const fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    #[must_use]
    pub const fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    #[must_use]
    pub const fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

impl Index {
    #[must_use]
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            unique: false,
        }
    }

    #[must_use]
    pub const fn unique(mut self) -> Self {
        self.unique = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintKind {
    PrimaryKey,
    ForeignKey { table: String, column: String },
    Unique,
    Check { expression: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub name: String,
    pub kind: ConstraintKind,
}

impl Constraint {
    #[must_use]
    pub fn new(name: impl Into<String>, kind: ConstraintKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub indexes: Vec<Index>,
    pub constraints: Vec<Constraint>,
}

impl Table {
    #[must_use]
    pub fn new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Self {
        Self {
            name: name.into(),
            columns,
            indexes: Vec::new(),
            constraints: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }

    #[must_use]
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableChange {
    AddColumn(ColumnDef),
    DropColumn(String),
    RenameColumn { from: String, to: String },
    ModifyColumn(ColumnDef),
    AddIndex(Index),
    DropIndex(String),
    AddConstraint(Constraint),
    DropConstraint(String),
}
