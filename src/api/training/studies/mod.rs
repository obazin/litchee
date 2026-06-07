//! The Studies API: export and import study PGN, list studies, edit chapters.
//!
//! Reached through [`LichessClient::studies`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{
    LichessStudyChapter, LichessStudyChapterPlayer, LichessStudyImportResult, LichessStudyMetadata,
};

/// Form body for importing PGN into a study.
#[derive(Debug, Serialize)]
struct ImportForm<'a> {
    name: &'a str,
    pgn: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<&'a str>,
}

/// Accessor for the Studies API.
#[derive(Debug)]
pub struct StudiesApi<'a> {
    client: &'a LichessClient,
}

impl<'a> StudiesApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Exports one chapter as PGN. `GET /api/study/{studyId}/{chapterId}.pgn`
    pub async fn export_chapter_pgn(&self, study_id: &str, chapter_id: &str) -> Result<String> {
        let path = format!("/api/study/{study_id}/{chapter_id}.pgn");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Exports all chapters of a study as PGN. `GET /api/study/{studyId}.pgn`
    pub async fn export_study_pgn(&self, study_id: &str) -> Result<String> {
        let path = format!("/api/study/{study_id}.pgn");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Exports all of a user's studies as PGN.
    /// `GET /api/study/by/{username}/export.pgn`
    pub async fn export_all_pgn(&self, username: &str) -> Result<String> {
        let path = format!("/api/study/by/{username}/export.pgn");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams metadata for a user's studies. `GET /api/study/by/{username}`
    pub async fn list_metadata(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessStudyMetadata>>> {
        let path = format!("/api/study/by/{username}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Imports PGN as one or more chapters of a study.
    /// `POST /api/study/{studyId}/import-pgn`
    #[must_use]
    pub fn import_pgn(
        &self,
        study_id: &'a str,
        name: &'a str,
        pgn: &'a str,
    ) -> ImportPgnRequest<'a> {
        ImportPgnRequest::new(self.client, study_id, name, pgn)
    }

    /// Deletes a chapter from a study.
    /// `DELETE /api/study/{studyId}/{chapterId}`
    pub async fn delete_chapter(&self, study_id: &str, chapter_id: &str) -> Result<()> {
        let path = format!("/api/study/{study_id}/{chapter_id}");
        http::ok(self.client.request(Method::DELETE, Host::Default, &path)).await
    }

    /// Starts building a new (empty) study. `POST /api/study`
    #[must_use]
    pub fn create_study(&self, name: &'a str) -> CreateStudyRequest<'a> {
        CreateStudyRequest::new(self.client, name)
    }

    /// Replaces the moves of a chapter from PGN.
    /// `POST /api/study/{studyId}/{chapterId}/moves`
    pub async fn update_chapter_moves(
        &self,
        study_id: &str,
        chapter_id: &str,
        pgn: &str,
    ) -> Result<()> {
        let path = format!("/api/study/{study_id}/{chapter_id}/moves");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("pgn", pgn)]);
        http::ok(request).await
    }

    /// Updates the PGN tags of a chapter.
    /// `POST /api/study/{studyId}/{chapterId}/tags`
    pub async fn update_chapter_tags(
        &self,
        study_id: &str,
        chapter_id: &str,
        pgn_tags: &str,
    ) -> Result<()> {
        let path = format!("/api/study/{study_id}/{chapter_id}/tags");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("pgn", pgn_tags)]);
        http::ok(request).await
    }
}

/// The id returned when creating a study.
#[derive(Debug, Deserialize)]
struct StudyCreated {
    id: String,
}

/// Form body for creating a study.
#[derive(Debug, Serialize)]
struct CreateStudyForm<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    computer: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explorer: Option<&'a str>,
}

/// Builder for creating a study.
#[derive(Debug)]
pub struct CreateStudyRequest<'a> {
    client: &'a LichessClient,
    form: CreateStudyForm<'a>,
}

impl<'a> CreateStudyRequest<'a> {
    /// Creates the request builder.
    fn new(client: &'a LichessClient, name: &'a str) -> Self {
        Self {
            client,
            form: CreateStudyForm {
                name,
                visibility: None,
                computer: None,
                explorer: None,
            },
        }
    }

    /// Sets the visibility (`public`, `unlisted`, or `private`).
    #[must_use]
    pub fn visibility(mut self, visibility: &'a str) -> Self {
        self.form.visibility = Some(visibility);
        self
    }

    /// Sets who may use computer analysis (`everyone`/`contributor`/`nobody`).
    #[must_use]
    pub fn computer(mut self, computer: &'a str) -> Self {
        self.form.computer = Some(computer);
        self
    }

    /// Sets who may use the opening explorer.
    #[must_use]
    pub fn explorer(mut self, explorer: &'a str) -> Self {
        self.form.explorer = Some(explorer);
        self
    }

    /// Creates the study, returning its id.
    pub async fn send(self) -> Result<String> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/study")
            .form(&self.form);
        let created: StudyCreated = http::json(request, "study id").await?;
        Ok(created.id)
    }
}

/// Builder for importing PGN into a study.
#[derive(Debug)]
pub struct ImportPgnRequest<'a> {
    client: &'a LichessClient,
    study_id: &'a str,
    form: ImportForm<'a>,
}

impl<'a> ImportPgnRequest<'a> {
    /// Creates the request builder.
    fn new(client: &'a LichessClient, study_id: &'a str, name: &'a str, pgn: &'a str) -> Self {
        Self {
            client,
            study_id,
            form: ImportForm {
                name,
                pgn,
                orientation: None,
                variant: None,
            },
        }
    }

    /// Sets the board orientation (`white` or `black`).
    #[must_use]
    pub fn orientation(mut self, orientation: &'a str) -> Self {
        self.form.orientation = Some(orientation);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: &'a str) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Performs the import.
    pub async fn send(self) -> Result<LichessStudyImportResult> {
        let path = format!("/api/study/{}/import-pgn", self.study_id);
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessStudyImportResult").await
    }
}

impl LichessClient {
    /// Studies API: export and import study PGN, list studies, edit chapters.
    #[must_use]
    pub fn studies(&self) -> StudiesApi<'_> {
        StudiesApi::new(self)
    }
}
