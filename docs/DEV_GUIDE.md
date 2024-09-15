# Developer Guide: Adding a New API Route and Database Table

This guide demonstrates how to apply the best practices outlined in our [Contributing Guide](CONTRIBUTING.md) by walking you through the process of adding a new API route and a corresponding database table with queries. We'll cover the entire workflow, from planning to submitting a pull request, ensuring that the new feature is bug-free, efficient, and adheres to our project's standards.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Scenario Overview](#scenario-overview)
3. [Step 1: Planning and Issue Tracking](#step-1-planning-and-issue-tracking)
4. [Step 2: Setting Up the Development Environment](#step-2-setting-up-the-development-environment)
5. [Step 3: Creating a Feature Branch](#step-3-creating-a-feature-branch)
6. [Step 4: Modifying the Database Schema](#step-4-modifying-the-database-schema)
    - [Adding a New Migration](#adding-a-new-migration)
    - [Updating Database Models](#updating-database-models)
7. [Step 5: Implementing Database Queries](#step-5-implementing-database-queries)
8. [Step 6: Updating Shared Libraries](#step-6-updating-shared-libraries)
9. [Step 7: Implementing the API Route in the REST Service](#step-7-implementing-the-api-route-in-the-rest-service)
    - [Adding Route Handlers](#adding-route-handlers)
    - [Registering the Route](#registering-the-route)
10. [Step 8: Writing Tests](#step-8-writing-tests)
    - [Unit Tests](#unit-tests)
    - [Integration Tests](#integration-tests)
11. [Step 9: Ensuring Code Quality](#step-9-ensuring-code-quality)
    - [Formatting and Linting](#formatting-and-linting)
    - [Static Analysis](#static-analysis)
12. [Step 10: Documentation](#step-10-documentation)
13. [Step 11: Committing and Pushing Changes](#step-11-committing-and-pushing-changes)
14. [Step 12: Submitting a Pull Request](#step-12-submitting-a-pull-request)
15. [Conclusion](#conclusion)
16. [Additional Resources](#additional-resources)

---

## Introduction

In this guide, we'll add a new feature to the **REST service**: an endpoint to manage "Projects". We'll create a new database table for projects, update the shared libraries with new models, implement database queries, and expose API routes for creating and retrieving projects. We'll follow best practices for development, testing, and code quality to ensure our contribution is efficient and reliable.

---

## Scenario Overview

- **Feature**: Add API endpoints to create and retrieve projects.
- **Database Changes**: Introduce a new `projects` table.
- **Components Affected**:
  - **Libraries**:
    - `libs/db`: New models and database interactions.
    - `libs/common`: Any shared models or utilities (if needed).
  - **Services**:
    - `services/rest_service`: New API routes and handlers.

---

## Step 1: Planning and Issue Tracking

- **Create an Issue**: Open a GitHub issue titled "Add Project Management API".
- **Define the Scope**:
  - **Endpoints**:
    - `POST /api/projects`: Create a new project.
    - `GET /api/projects`: Retrieve a list of projects.
  - **Database**:
    - New `projects` table with fields `id`, `name`, `description`, `created_at`, and `updated_at`.
- **Assign the Issue**: Assign yourself to the issue to indicate you are working on it.

---

## Step 2: Setting Up the Development Environment

- **Ensure Tools Are Installed**:
  - Rust Toolchain (`rustup`)
  - `cargo-edit`, `cargo-watch`, `cargo-nextest`, `clippy`, `rustfmt`, `cargo-audit`
- **Update Dependencies**:

  ```bash
  cd agentserver.rs
  cargo update
  ```

- **Start the Development Database**: Ensure your PostgreSQL instance is running and accessible.

---

## Step 3: Creating a Feature Branch

```bash
git checkout -b feature/project-management develop
```

---

## Step 4: Modifying the Database Schema

### Adding a New Migration

We'll use `sqlx`'s migration tool to create a new migration.

1. **Navigate to the `libs/db` Directory**:

   ```bash
   cd libs/db
   ```

2. **Create Migration File**:

   ```bash
   sqlx migrate add create_projects_table
   ```

   This creates a new SQL file in `libs/db/migrations/`.

3. **Edit Migration File**: Open the newly created migration file and add the SQL statements.

   ```sql
   -- libs/db/migrations/{timestamp}__create_projects_table.sql

   CREATE TABLE projects (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL,
       description TEXT,
       created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
       updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
   );
   ```

4. **Apply Migrations**: Run migrations against your development database.

   ```bash
   sqlx migrate run
   ```

### Updating Database Models

1. **Create a Project Model**:

   ```rust
   // libs/db/src/models/project.rs

   use serde::{Deserialize, Serialize};
   use uuid::Uuid;
   use chrono::{DateTime, Utc};
   use sqlx::FromRow;

   #[derive(Debug, Serialize, Deserialize, FromRow)]
   pub struct Project {
       pub id: Uuid,
       pub name: String,
       pub description: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```

2. **Update the Models Module**:

   ```rust
   // libs/db/src/models/mod.rs

   pub mod project;

   pub use project::Project;

   // ... existing models
   ```

---

## Step 5: Implementing Database Queries

Implement database functions in the `libs/db` crate.

```rust
// libs/db/src/queries/project_queries.rs

use crate::models::Project;
use sqlx::PgPool;
use anyhow::Result;

pub async fn create_project(pool: &PgPool, name: &str, description: Option<&str>) -> Result<Project> {
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

pub async fn get_projects(pool: &PgPool) -> Result<Vec<Project>> {
    let projects = sqlx::query_as::<_, Project>(
        r#"
        SELECT * FROM projects ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(projects)
}
```

- **Update the Queries Module**:

  ```rust
  // libs/db/src/queries/mod.rs

  pub mod project_queries;

  pub use project_queries::{create_project, get_projects};

  // ... existing queries
  ```

- **Best Practices Applied**:
  - **Use of `sqlx`**: For compile-time query checking.
  - **Error Handling**: Using `anyhow::Result` for simplicity.
  - **Async/Await**: For asynchronous database operations.

---

## Step 6: Updating Shared Libraries

1. **Update `libs/db/src/lib.rs`**:

   ```rust
   // libs/db/src/lib.rs

   pub mod models;
   pub mod queries;
   pub mod connection; // If you have a connection module
   // ... other modules
   ```

2. **Ensure Proper Exports**:

   - Re-export structs or functions if necessary for easier access in services.

3. **Add Dependencies in `services/rest_service/Cargo.toml`**:

   ```toml
   [dependencies]
   db = { path = "../../libs/db" }
   common = { path = "../../libs/common" } # If needed
   # ... other dependencies
   ```

---

## Step 7: Implementing the API Route in the REST Service

### Adding Route Handlers

1. **Create Handler Functions**:

   ```rust
   // services/rest_service/src/handlers/projects.rs

   use axum::{
       extract::{Extension, Json},
       http::StatusCode,
   };
   use db::queries::project_queries::{create_project, get_projects};
   use db::models::Project;
   use sqlx::PgPool;
   use serde::Deserialize;
   use anyhow::Result;
   use tracing::instrument;

   #[derive(Deserialize)]
   pub struct CreateProjectInput {
       pub name: String,
       pub description: Option<String>,
   }

   #[instrument(skip(pool))]
   pub async fn create_project_handler(
       Json(input): Json<CreateProjectInput>,
       Extension(pool): Extension<PgPool>,
   ) -> Result<(StatusCode, Json<Project>), (StatusCode, String)> {
       let project = create_project(&pool, &input.name, input.description.as_deref())
           .await
           .map_err(internal_error)?;

       Ok((StatusCode::CREATED, Json(project)))
   }

   #[instrument(skip(pool))]
   pub async fn list_projects_handler(
       Extension(pool): Extension<PgPool>,
   ) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
       let projects = get_projects(&pool).await.map_err(internal_error)?;

       Ok(Json(projects))
   }

   fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
       (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
   }
   ```

   - **Best Practices Applied**:
     - **Error Handling**: Converting errors into HTTP responses.
     - **Logging**: Using `tracing::instrument` to log function calls and parameters.
     - **Deserialization**: Using `serde` for JSON payloads.
     - **Dependency Injection**: Using `Extension` to pass the database pool.

2. **Update the Handlers Module**:

   ```rust
   // services/rest_service/src/handlers/mod.rs

   pub mod projects;

   // ... existing handlers
   ```

### Registering the Route

1. **Update the Routes Module**:

   ```rust
   // services/rest_service/src/routes.rs

   use axum::{
       routing::{get, post},
       Router,
   };
   use crate::handlers::projects::{create_project_handler, list_projects_handler};

   pub fn create_router() -> Router {
       Router::new()
           .route("/api/projects", post(create_project_handler).get(list_projects_handler))
           // ... existing routes
   }
   ```

2. **Ensure Routes Are Mounted in `main.rs`**:

   ```rust
   // services/rest_service/src/main.rs

   use axum::{Router, Extension};
   use sqlx::postgres::PgPoolOptions;
   use crate::routes::create_router;
   use tracing_subscriber;

   #[tokio::main]
   async fn main() {
       // Initialize tracing
       tracing_subscriber::fmt::init();

       // Create database pool
       let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
       let pool = PgPoolOptions::new()
           .max_connections(5)
           .connect(&database_url)
           .await
           .expect("Failed to create pool");

       // Build our application with a route
       let app = create_router().layer(Extension(pool));

       // Run it
       axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
           .serve(app.into_make_service())
           .await
           .unwrap();
   }
   ```

   - **Best Practices Applied**:
     - **Environment Variables**: Using `DATABASE_URL` from the environment.
     - **Connection Pooling**: Managing database connections efficiently.
     - **Layering Extensions**: Making the database pool available to handlers.

---

## Step 8: Writing Tests

### Unit Tests

1. **Test Database Functions**:

   ```rust
   // libs/db/src/queries/project_queries.rs

   #[cfg(test)]
   mod tests {
       use super::*;
       use sqlx::{Pool, Postgres, PgPoolOptions};
       use config::{Config, File, Environment};

       async fn setup_db() -> Pool<Postgres> {
           let config = Config::builder()
               .add_source(File::with_name("config/test"))
               .add_source(Environment::with_prefix("APP"))
               .build()
               .expect("Failed to load configuration");

           let database_url = config.get_string("database_url").expect("DATABASE_URL must be set in config");
           PgPoolOptions::new()
               .max_connections(1)
               .connect(&database_url)
               .await
               .expect("Failed to connect to test database")
       }

       #[tokio::test]
       async fn test_create_and_get_projects() {
           let pool = setup_db().await;
           // Clean up the projects table before testing
           sqlx::query("DELETE FROM projects").execute(&pool).await.unwrap();

           // Test create_project
           let project = create_project(&pool, "Test Project", Some("Description"))
               .await
               .unwrap();
           assert_eq!(project.name, "Test Project");

           // Test get_projects
           let projects = get_projects(&pool).await.unwrap();
           assert_eq!(projects.len(), 1);
           assert_eq!(projects[0].name, "Test Project");
       }
   }
   ```

   - **Best Practices Applied**:
     - **Isolated Tests**: Using a test database to avoid interfering with production data.
     - **Cleanup**: Ensuring the database is in a known state before testing.
     - **Async Testing**: Using `#[tokio::test]` for asynchronous tests.

### Integration Tests

1. **Create Integration Test File**:

   ```rust
   // services/rest_service/tests/projects_integration_tests.rs

   use axum::{
       body::Body,
       http::{Request, StatusCode},
   };
   use tower::ServiceExt; // for `oneshot`
   use serde_json::json;
   use db::models::Project;
   use crate::main::create_router;

   #[tokio::test]
   async fn test_create_and_list_projects_api() {
       // Set up the app
       let app = create_router();

       // Test POST /api/projects
       let response = app
           .oneshot(
               Request::builder()
                   .method("POST")
                   .uri("/api/projects")
                   .header("Content-Type", "application/json")
                   .body(Body::from(
                       serde_json::to_vec(&json!({
                           "name": "API Test Project",
                           "description": "Test Description"
                       }))
                       .unwrap(),
                   ))
                   .unwrap(),
           )
           .await
           .unwrap();

       assert_eq!(response.status(), StatusCode::CREATED);

       // Test GET /api/projects
       let response = app
           .oneshot(
               Request::builder()
                   .method("GET")
                   .uri("/api/projects")
                   .body(Body::empty())
                   .unwrap(),
           )
           .await
           .unwrap();

       assert_eq!(response.status(), StatusCode::OK);

       let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
       let projects: Vec<Project> = serde_json::from_slice(&body).unwrap();
       assert_eq!(projects.len(), 1);
       assert_eq!(projects[0].name, "API Test Project");
   }
   ```

   - **Best Practices Applied**:
     - **Integration Testing**: Testing the entire request-response cycle.
     - **Using `oneshot`**: To send a request to the app without starting a server.
     - **Deserializing Responses**: To verify the correctness of API outputs.

---

## Step 9: Ensuring Code Quality

### Formatting and Linting

1. **Run Rustfmt**:

   ```bash
   cargo fmt --all
   ```

2. **Run Clippy**:

   ```bash
   cargo clippy --all -- -D warnings
   ```

   - **Best Practices Applied**:
     - **Consistency**: Ensuring code adheres to style guidelines.
     - **Quality**: Catching potential issues early.

### Static Analysis

1. **Run Cargo Audit**:

   ```bash
   cargo audit
   ```

2. **Check for Outdated Dependencies**:

   ```bash
   cargo outdated
   ```

   - **Best Practices Applied**:
     - **Security**: Identifying vulnerabilities.
     - **Maintenance**: Keeping dependencies up to date.

---

## Step 10: Documentation

1. **Document Public Functions and Structs**:

   ```rust
   /// Creates a new project in the database.
   ///
   /// # Arguments
   ///
   /// * `pool` - A reference to the PostgreSQL connection pool.
   /// * `name` - The name of the project.
   /// * `description` - An optional description of the project.
   ///
   /// # Returns
   ///
   /// * `Project` - The newly created project.
   pub async fn create_project(/* ... */) -> Result<Project> { /* ... */ }
   ```

2. **Update README**: If necessary, update any relevant sections in the `README.md`.

3. **Generate Documentation**:

   ```bash
   cargo doc --open
   ```

   - **Best Practices Applied**:
     - **Clarity**: Helping other developers understand your code.
     - **Maintainability**: Easier future updates and bug fixes.

---

## Step 11: Committing and Pushing Changes

1. **Add Changes**:

   ```bash
   git add .
   ```

2. **Write Descriptive Commit Messages**:

   ```bash
   git commit -m "feat(rest_service): add project management API endpoints"
   ```

   - **Follow Conventional Commits**:
     - **feat**: A new feature.
     - **fix**: A bug fix.
     - **docs**: Documentation changes.
     - **style**: Code style changes (formatting, missing semi-colons, etc.).
     - **refactor**: Code changes that neither fix a bug nor add a feature.

3. **Push to Remote Branch**:

   ```bash
   git push origin feature/project-management
   ```

---

## Step 12: Submitting a Pull Request

1. **Open Pull Request**:

   - **Title**: `feat(rest_service): add project management API endpoints`
   - **Description**:
     - Briefly explain what changes you've made.
     - Reference the issue number (e.g., "Closes #123").
     - Highlight any important points or considerations.

2. **Ensure CI Passes**:

   - Wait for Continuous Integration checks to pass.
   - Address any issues if they arise.

3. **Review Feedback**:

   - Respond promptly to code review comments.
   - Make necessary changes and push updates.

---

## Conclusion

By following this guide, you've successfully added a new API route and database table while adhering to our project's structure and best practices. You've utilized the recommended tools, written tests to ensure correctness, and maintained high code quality throughout the process.

- **Project Structure Alignment**: Ensured that all code additions fit within the established `libs` and `services` directories.
- **Best Practices**: Followed Rust conventions and project guidelines for code quality, testing, and documentation.
- **Modularity**: Leveraged shared libraries in `libs/` for reusability and maintainability.
- **Rapid Development**: Structured the project to facilitate quick and efficient development.

---

## Additional Resources

- **Contributing Guide**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Rust Book**: [The Rust Programming Language](https://doc.rust-lang.org/book/)
- **Tokio Documentation**: [Tokio Async Runtime](https://tokio.rs/tokio/tutorial)
- **Axum Documentation**: [Axum Web Framework](https://docs.rs/axum)
- **SQLx Documentation**: [SQLx Async Rust SQL crate](https://docs.rs/sqlx)
- **Project Structure Overview**: [Project Structure Documentation](PROJECT_STRUCTURE.md) *(if available)*

---

**Note**: Remember to keep your feature branch up-to-date with the `develop` branch by rebasing or merging as necessary. This ensures that your code integrates smoothly with the latest changes in the codebase.

If you have any questions or need assistance, don't hesitate to reach out to the project maintainers or open a discussion in the project's communication channels.