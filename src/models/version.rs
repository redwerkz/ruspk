use crate::models::DbPackage;
use crate::schema::*;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Queryable, Associations, Identifiable, Debug, Clone)]
#[belongs_to(DbPackage, foreign_key = "package_id")]
#[table_name = "version"]
pub struct DbVersion {
    pub id: u64,
    pub package_id: u64,
    pub ver: u32,
    pub upstream_version: String,
    pub changelog: Option<String>,
    pub report_url: Option<String>,
    pub distributor: Option<String>,
    pub distributor_url: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_url: Option<String>,
    pub dependencies: Option<String>,
    pub conf_dependencies: Option<String>,
    pub conflicts: Option<String>,
    pub conf_conflicts: Option<String>,
    pub install_wizard: Option<bool>,
    pub upgrade_wizard: Option<bool>,
    pub startable: Option<bool>,
    pub license: Option<String>,
    pub insert_date: NaiveDateTime,
}