use crate::schema::Schema;
use std::sync::Arc;

pub fn create_test_schema() -> Arc<Schema> {
    Arc::new(
        Schema::parse(
            r#"
        namespaces:
          - name: exocore
            traits:
              - id: 0
                name: contact
                id_field:
                    field: 0
                fields:
                  - id: 0
                    name: id
                    type: string
                    indexed: true
                  - id: 1
                    name: name
                    type: string
                    indexed: true
                  - id: 2
                    name: email
                    type: string
                    indexed: true
              - id: 1
                name: email
                id_field: specified
                fields:
                  - id: 0
                    name: subject
                    type: string
                    indexed: true
                  - id: 1
                    name: body
                    type: string
                    indexed: true
                  - id: 2
                    name: from
                    type:
                        struct: 0
                    indexed: true
              - id: 2
                name: note
                id_field: generated
                fields:
                  - id: 0
                    name: title
                    type: string
                    indexed: true
                  - id: 1
                    name: body
                    type: string
                    indexed: true
              - id: 3
                name: annotation
                id_field: generated
                fields:
                  - id: 0
                    name: count
                    type: int
                    indexed: false
              - id: 4
                name: collection
                id_field:
                    static: collection_id
                fields:
                  - id: 0
                    name: name
                    type: string
                    indexed: true
              - id: 5
                name: combined_id
                id_field:
                    fields:
                      - 0
                      - 1
                fields:
                  - id: 0
                    name: id1
                    type: string
                    indexed: true
                  - id: 1
                    name: id2
                    type: string
                    indexed: true
            structs:
              - id: 0
                name: email_contact
                fields:
                  - id: 0
                    name: name
                    type: string
                    indexed: true
                  - id: 1
                    name: email
                    type: string
                    indexed: true

        "#,
        )
        .unwrap(),
    )
}
