use crate::error::AppError;
use crate::ir::{Column, Diagram, EndPoint, RelationType, Relationship, Table, TableId};

/// Parse a DBML string into a Diagram.
pub fn parse_dbml(input: &str) -> Result<Diagram, AppError> {
    let schema = dbml_rs::parse_dbml(input).map_err(|e| AppError::ParseError(format!("{}", e)))?;

    // Get Tables from parsed schema
    let tables: Vec<Table> = schema
        .tables()
        .iter()
        .map(|t| {
            let schema_name = t
                .ident
                .schema
                .as_ref()
                .map(|s| s.to_string.clone())
                .unwrap_or_else(|| "public".to_string());
            let name = t.ident.name.to_string.clone();
            let columns: Vec<Column> = t
                .cols
                .iter()
                .map(|c| {
                    let is_pk = c.settings.as_ref().map(|s| s.is_pk).unwrap_or(false);
                    let is_nullable = c
                        .settings
                        .as_ref()
                        .and_then(|s| s.nullable.as_ref())
                        .map(|n| matches!(n, dbml_rs::ast::Nullable::Null))
                        .unwrap_or(true);
                    Column {
                        name: c.name.to_string.clone(),
                        type_raw: c.r#type.raw.clone(),
                        is_pk,
                        is_nullable,
                    }
                })
                .collect();

            Table {
                id: TableId::new(schema_name, name),
                columns,
                position: None,
            }
        })
        .collect();

    // Get Relationships from parsed schema
    let mut relationships: Vec<Relationship> = Vec::new();
    for r in schema.refs() {
        let relation_type = match r.rel {
            dbml_rs::ast::Relation::One2One => RelationType::OneToOne,
            dbml_rs::ast::Relation::One2Many => RelationType::OneToMany,
            dbml_rs::ast::Relation::Many2One => RelationType::ManyToOne,
            dbml_rs::ast::Relation::Many2Many => RelationType::ManyToMany,
            dbml_rs::ast::Relation::Undef => continue,
        };

        // Create EndPoint from Left-Hand-Side
        // e.g. posts.user_id > users.id -> posts.user_id
        let from_schema = r
            .lhs
            .schema
            .as_ref()
            .map(|s| s.to_string.clone())
            .unwrap_or_else(|| "public".to_string());
        let from_table = r.lhs.table.to_string.clone();
        let from_cols: Vec<String> = r
            .lhs
            .compositions
            .iter()
            .map(|c| c.to_string.clone())
            .collect();

        // Create EndPoint from Right-Hand-Side
        // e.g. posts.user_id > users.id -> users.id
        let to_schema = r
            .rhs
            .schema
            .as_ref()
            .map(|s| s.to_string.clone())
            .unwrap_or_else(|| "public".to_string());
        let to_table = r.rhs.table.to_string.clone();
        let to_cols: Vec<String> = r
            .rhs
            .compositions
            .iter()
            .map(|c| c.to_string.clone())
            .collect();

        relationships.push(Relationship {
            relation_type,
            from: EndPoint {
                table_id: TableId::new(from_schema, from_table),
                column_names: from_cols,
            },
            to: EndPoint {
                table_id: TableId::new(to_schema, to_table),
                column_names: to_cols,
            },
        });
    }

    // Retrieve inline refs from each table
    for t in schema.tables() {
        let schema_name = t
            .ident
            .schema
            .as_ref()
            .map(|s| s.to_string.clone())
            .unwrap_or_else(|| "public".to_string());
        let table_name = t.ident.name.to_string.clone();
        for col in &t.cols {
            if let Some(settings) = &col.settings {
                for inline_ref in &settings.refs {
                    let relation_type = match inline_ref.rel {
                        dbml_rs::ast::Relation::One2One => RelationType::OneToOne,
                        dbml_rs::ast::Relation::One2Many => RelationType::OneToMany,
                        dbml_rs::ast::Relation::Many2One => RelationType::ManyToOne,
                        dbml_rs::ast::Relation::Many2Many => RelationType::ManyToMany,
                        dbml_rs::ast::Relation::Undef => continue,
                    };
                    let from = EndPoint {
                        table_id: TableId::new(&schema_name, &table_name),
                        column_names: vec![col.name.to_string.clone()],
                    };
                    let to_schema = inline_ref
                        .rhs
                        .schema
                        .as_ref()
                        .map(|s| s.to_string.clone())
                        .unwrap_or_else(|| "public".to_string());
                    let to_table = inline_ref.rhs.table.to_string.clone();
                    let to_cols: Vec<String> = inline_ref
                        .rhs
                        .compositions
                        .iter()
                        .map(|c| c.to_string.clone())
                        .collect();
                    let to = EndPoint {
                        table_id: TableId::new(to_schema, to_table),
                        column_names: to_cols,
                    };
                    relationships.push(Relationship {
                        relation_type,
                        from: from,
                        to: to,
                    });
                }
            }
        }
    }
    Ok(Diagram {
        tables,
        relationships,
    })
}
