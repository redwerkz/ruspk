use crate::models::DbArchitecture;
use crate::models::DbLanguage;
use crate::schema::*;
use crate::Connection;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::query_builder::SqlQuery;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Integer, Nullable, Text};
#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug, Clone)]
#[table_name = "package"]
pub struct DbPackage {
    pub id: i64,
    pub author_user_id: Option<i64>,
    pub name: String,
    pub insert_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct Package {
    pub id: i64,
    pub author: String,
    pub name: String,
    pub insert_date: Option<NaiveDateTime>,
}

impl DbPackage {
    pub fn find_all(conn: &Connection, limit: i64, offset: i64) -> QueryResult<Vec<Package>> {
        package::table
            .order(package::id.desc())
            .limit(limit)
            .offset(offset)
            .inner_join(user::table)
            .select((package::id, user::username, package::name, package::insert_date))
            .load::<Package>(conn)
    }

    pub fn create_package(conn: &Connection, author_id: Option<i64>, name: String) -> QueryResult<DbPackage> {
        let new_package = (
            package::author_user_id.eq(author_id),
            package::name.eq(name),
            package::insert_date.eq(dsl::now),
        );

        let package = diesel::insert_into(package::table)
            .values(&new_package)
            .get_result::<DbPackage>(conn)?;
        Ok(package)
    }

    pub fn delete_package(conn: &Connection, id: i64) -> QueryResult<usize> {
        conn.build_transaction().read_write().run(|| {
            let builds = diesel::delete(build::table.filter(
                build::version_id.eq_any(version::table.filter(version::package_id.eq(id)).select(version::id)),
            ))
            .execute(conn)?;

            let versions = diesel::delete(version::table.filter(version::package_id.eq(id))).execute(conn)?;

            let packages = diesel::delete(package::table.filter(package::id.eq(id))).execute(conn)?;

            Ok(builds + versions + packages) // number of rows effected
        })
    }

    pub fn get_packages(
        lang: &str,
        arch: &str,
        build: i64,
        beta: bool,
        _major: i8,
        _micro: i8,
        _minor: i8,
        conn: &Connection,
    ) -> Result<Vec<DBQueryResultPackage>> {
        let language_id = DbLanguage::get_language_id(conn, lang);
        let architecture_id = DbArchitecture::get_architecture_id(conn, arch)?; // todo return 404

        let query = sql_query(
            r#"
                SELECT
                `package`.`id` AS package_id,
                `version`.`id` AS version_id,
                (CASE WHEN `version`.`report_url` <> '' THEN true ELSE false END) AS beta,
                `version`.`conflicts` AS conflictpkgs,
                `version`.`dependencies` AS deppkgs,
                `version`.`changelog`,
                `description`.`description` AS "desc",
                `version`.`distributor`,
                `version`.`distributor_url`,
                `displayname`.`displayname` AS dname,
                `build`.`path` AS link,
                `version`.`maintainer`,
                `version`.`maintainer_url`,
                `package`.`name` AS package,
                `version`.`install_wizard` AS qinst,
                `version`.`startable` AS qstart,
                `version`.`upgrade_wizard` AS qupgrade,
                `version`.`upstream_version`,
                `version`.`version` AS revision,
                `build`.`md5`,
                `build`.`extract_size` AS size

                FROM
                (
                    (
                    `package`
                    INNER JOIN (
                        (
                        (
                            `version`
                            LEFT OUTER JOIN `description` ON `description`.`version_id` = `version`.`id`
                            AND `description`.`language_id` = CASE WHEN EXISTS (
                            SELECT 1
                            FROM `description`
                            WHERE `description`.`language_id` = ?
                            ) THEN ? ELSE 1 END
                        )
                        LEFT OUTER JOIN `displayname` ON `displayname`.`version_id` = `version`.`id`
                        AND `displayname`.`language_id` = CASE WHEN EXISTS (
                            SELECT  1
                            FROM  `displayname`
                            WHERE  `displayname`.`language_id` = ?
                        ) THEN ? ELSE 1 END
                        )
                        INNER JOIN (
                        SELECT MAX(`version`.`version`) `version`, `package_id`
                        FROM `version`
                        GROUP BY `version`.`package_id`
                        ) ver ON `version`.`package_id` = `ver`.`package_id`
                        AND `version`.`version` = `ver`.`version`
                    ) ON `version`.`package_id` = `package`.`id`
                    )
                    INNER JOIN (
                    (
                        `build`
                        INNER JOIN `firmware` ON `firmware`.`id` = `build`.`firmware_id`s
                    )
                    INNER JOIN `build_architecture` ON `build_architecture`.`build_id` = `build`.`id`
                    AND `build_architecture`.`architecture_id` IN(1, ?)
                    ) ON `build`.`version_id` = `version`.`id`
                )
                WHERE `build`.`active` = true
                AND `firmware`.`build` <= ?
                AND (? OR `version`.`report_url` = '')
            "#,
        );
        let packages = bind_and_load(conn, query, language_id, architecture_id, build, beta)?;
        Ok(packages)
    }
}

pub fn bind_and_load(
    conn: &Connection,
    query: SqlQuery,
    language_id: i64,
    architecture_id: i64,
    build: i64,
    beta: bool,
) -> Result<Vec<DBQueryResultPackage>> {
    let result = query
        .bind::<BigInt, _>(language_id)
        .bind::<BigInt, _>(language_id)
        .bind::<BigInt, _>(language_id)
        .bind::<BigInt, _>(language_id)
        .bind::<BigInt, _>(architecture_id)
        .bind::<BigInt, _>(build)
        .bind::<Bool, _>(beta)
        .load::<DBQueryResultPackage>(conn)
        .context("Error loading packages from DB")?;
    Ok(result)
}

#[derive(Serialize, QueryableByName, Debug, Clone)]
pub struct DBQueryResultPackage {
    #[sql_type = "BigInt"]
    pub package_id: i64,
    #[sql_type = "BigInt"]
    pub version_id: i64,
    #[sql_type = "Bool"]
    pub beta: bool,
    #[sql_type = "Nullable<Text>"]
    pub conflictpkgs: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub deppkgs: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub changelog: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub desc: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub distributor: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub distributor_url: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub dname: Option<String>,
    // download_count: u64,
    #[sql_type = "Nullable<Text>"]
    pub link: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub maintainer: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub maintainer_url: Option<String>,
    #[sql_type = "Text"]
    pub package: String,
    #[sql_type = "Nullable<Bool>"]
    pub qinst: Option<bool>,
    #[sql_type = "Nullable<Bool>"]
    pub qstart: Option<bool>,
    #[sql_type = "Nullable<Bool>"]
    pub qupgrade: Option<bool>,
    // recent_download_count: u64,
    #[sql_type = "Text"]
    pub upstream_version: String,
    #[sql_type = "Integer"]
    pub revision: i32,
    #[sql_type = "Nullable<Text>"]
    pub md5: Option<String>,
    #[sql_type = "Nullable<Integer>"]
    pub size: Option<i32>,
}
