use crate::{
    charts::{insert::DateValue, updater::ChartPartialUpdater},
    UpdateError,
};
use async_trait::async_trait;
use entity::sea_orm_active_enums::ChartType;
use sea_orm::{prelude::*, DbBackend, FromQueryResult, Statement};

#[derive(Default, Debug)]
pub struct NewVerifiedContracts {}

#[async_trait]
impl ChartPartialUpdater for NewVerifiedContracts {
    async fn get_values(
        &self,
        blockscout: &DatabaseConnection,
        last_row: Option<DateValue>,
    ) -> Result<Vec<DateValue>, UpdateError> {
        let stmnt = match last_row {
            Some(row) => Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT
                    DATE(b.timestamp) as date,
                    COUNT(*)::TEXT as value
                FROM transactions t
                JOIN blocks       b ON b.hash = t.block_hash
                JOIN addresses    a ON t.created_contract_address_hash = a.hash
                WHERE
                    LENGTH(t.created_contract_address_hash) > 0 AND
                    b.consensus = true AND
                    a.verified = true AND
                    DATE(b.timestamp) > $1
                GROUP BY DATE(b.timestamp)"#,
                vec![row.date.into()],
            ),
            None => Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT
                    DATE(b.timestamp) as date,
                    COUNT(*)::TEXT as value
                FROM transactions t
                JOIN blocks       b ON b.hash = t.block_hash
                JOIN addresses    a ON t.created_contract_address_hash = a.hash
                WHERE
                    LENGTH(t.created_contract_address_hash) > 0 AND
                    b.consensus = true AND
                    a.verified = true
                GROUP BY DATE(b.timestamp)"#,
                vec![],
            ),
        };

        let data = DateValue::find_by_statement(stmnt)
            .all(blockscout)
            .await
            .map_err(UpdateError::BlockscoutDB)?;
        Ok(data)
    }
}

#[async_trait]
impl crate::Chart for NewVerifiedContracts {
    fn name(&self) -> &str {
        "newVerifiedContracts"
    }

    fn chart_type(&self) -> ChartType {
        ChartType::Line
    }

    async fn update(
        &self,
        db: &DatabaseConnection,
        blockscout: &DatabaseConnection,
        force_full: bool,
    ) -> Result<(), UpdateError> {
        self.update_with_values(db, blockscout, force_full).await
    }
}

#[cfg(test)]
mod tests {
    use super::NewVerifiedContracts;
    use crate::tests::simple_test::simple_test_chart;

    #[tokio::test]
    #[ignore = "needs database to run"]
    async fn update_new_verified_contracts() {
        let chart = NewVerifiedContracts::default();
        simple_test_chart(
            "update_new_verified_contracts",
            chart,
            vec![("2022-11-10", "1"), ("2022-11-11", "2")],
        )
        .await;
    }
}
