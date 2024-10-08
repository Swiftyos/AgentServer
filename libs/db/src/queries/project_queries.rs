use crate::models::Project;
use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, instrument};

/// Creates a new project in the database.
///
/// # Arguments
///
/// * `pool` - A reference to the PostgreSQL connection pool.
/// * `name` - The name of the project.
/// * `description` - The description of the project.
///
/// # Returns
///
/// * `Result<Project>` - A Result containing the newly created Project if successful,
///   or an error if the query fails.
///
/// # Errors
///
/// This function will return an error if the database query fails to execute.
pub async fn create_project(pool: &PgPool, name: &str, description: &str) -> Result<Project> {
    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (name, description)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(project)
}

/// Retrieves a paginated list of projects from the database.
///
/// # Arguments
///
/// * `pool` - A reference to the PostgreSQL connection pool.
/// * `page` - The page number to retrieve (1-indexed).
/// * `page_size` - The number of projects to retrieve per page.
///
/// # Returns
///
/// * `Result<Vec<Project>>` - A Result containing a vector of Project structs if successful,
///   or an error if the query fails.
///
/// # Notes
///
/// Projects are ordered by creation date in descending order (newest first).
#[instrument(name = "db.get_projects", skip_all, fields(page, page_size))]
pub async fn get_projects(
    pool: &PgPool,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<Vec<Project>> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);

    let offset = (page - 1) * page_size;

    info!(
        "Fetching projects with page: {:?}, page_size: {:?}",
        page, page_size
    );
    let projects = sqlx::query_as::<_, Project>(
        r#"
        SELECT * FROM projects
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    info!("Fetched {:?} projects", projects.len());
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{apply_migrations, create_pool};
    use config::{Config, Environment, File};
    use sqlx::{Pool, Postgres};
    use tracing::{error, info};
    use tracing_test::traced_test;

    async fn setup_db() -> Pool<Postgres> {
        let config = Config::builder()
            .add_source(File::with_name("../../config/test.toml"))
            .add_source(Environment::with_prefix("APP"))
            .build()
            .expect("Failed to load configuration");

        let database_url = config
            .get_string("database_url")
            .expect("DATABASE_URL must be set in config");

        let schema_string = format!(
            "test_schema_{}",
            uuid::Uuid::new_v4().to_string().replace("-", "")
        );
        let schema = Some(schema_string.as_str());

        info!("Database URL: {}", database_url);
        info!("Schema: {}", schema_string);
        let pool = create_pool(&database_url, schema)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e));
        match pool {
            Ok(pool) => {
                apply_migrations(&pool).await.unwrap();
                pool
            }
            Err(e) => {
                error!("Failed to create database pool: {}", e);
                panic!("Failed to create database pool: {}", e);
            }
        }
    }

    #[traced_test]
    #[tokio::test]
    async fn test_create_and_get_projects() {
        let pool = setup_db().await;
        // Clean up the projects table before testing
        sqlx::query("TRUNCATE TABLE projects")
            .execute(&pool)
            .await
            .unwrap();

        // Test create_project
        let project = create_project(&pool, "Test Project", "Description")
            .await
            .unwrap();
        assert_eq!(project.name, "Test Project");

        // Test get_projects
        let projects = get_projects(&pool, Some(1), Some(10)).await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Test Project");
    }

    #[traced_test]
    #[tokio::test]
    async fn test_pagination() {
        let pool = setup_db().await;
        // Clean up the projects table before testing
        sqlx::query("TRUNCATE TABLE projects")
            .execute(&pool)
            .await
            .unwrap();

        // Create multiple projects
        for i in (1..=15).rev() {
            create_project(&pool, &format!("Project {}", i), "Description")
                .await
                .unwrap();
        }

        // Verify total number of projects
        let all_projects = sqlx::query_as::<_, Project>("SELECT * FROM projects")
            .fetch_all(&pool)
            .await
            .unwrap();
        info!("Total projects: {}", all_projects.len());
        assert_eq!(all_projects.len(), 15, "Expected 15 total projects");

        // Test first page
        let first_page = get_projects(&pool, Some(1), Some(10)).await.unwrap();
        assert_eq!(first_page.len(), 10);
        assert_eq!(first_page[0].name, "Project 1");
        assert_eq!(first_page[9].name, "Project 10");

        // Test second page
        let second_page = get_projects(&pool, Some(2), Some(10)).await.unwrap();

        assert_eq!(second_page[0].name, "Project 11");
        assert_eq!(second_page[4].name, "Project 15");
        assert_eq!(second_page.len(), 5);
        // Test empty page
        let empty_page = get_projects(&pool, Some(3), Some(10)).await.unwrap();
        assert_eq!(empty_page.len(), 0);
    }
}
