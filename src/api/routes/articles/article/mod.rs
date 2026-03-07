use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, patch, post, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::{OptionalAuth, RequireAdmin},
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{ArticleAuthor, ArticleDeleteResponse, ArticleRequest, ArticleResponse},
    repository::{BlogArticleUpdate, Repositories},
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get an article",
    description = "Retrieve an article by its ID",
    tag = "Articles"
)]
async fn get_article(
    auth: OptionalAuth,
    repos: Data<Repositories>,
    article_id: Path<String>,
) -> ApiResult<Json<ArticleResponse>> {
    let article_id = article_id.into_inner();

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Fetching article",
    );

    let article = repos
        .blog_articles
        .find_by_id(&article_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Article with ID {} not found", article_id)))?;

    let is_admin = auth.0.as_ref().is_some_and(|ctx| ctx.is_admin());

    if article.is_draft && !is_admin {
        return Err(ApiError::NotFound(format!(
            "Article with ID {} not found",
            article_id
        )));
    }

    let author = repos
        .users
        .find_by_id(&article.author_id)
        .await?
        .map(|user| ArticleAuthor {
            avatar: user.avatar,
            username: user.username,
        });

    Ok(Json(ArticleResponse::from_article(article, author)?))
}

#[api_operation(
    summary = "Publish an article",
    description = "Publish a new article with the provided information",
    tag = "Articles",
    skip
)]
async fn publish_article(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    article_id: Path<String>,
) -> ApiResult<Json<ArticleResponse>> {
    let article_id = article_id.into_inner();

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Publishing article",
    );

    let article = repos
        .blog_articles
        .find_by_id(&article_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Article with ID {} not found", article_id)))?;

    if !article.is_draft {
        return Err(ApiError::AlreadyPublished);
    }

    let update = BlogArticleUpdate::new().with_is_draft(false);

    let update_result = repos.blog_articles.update(&article_id, update).await?;

    let updated_article = update_result.ok_or_else(|| {
        ApiError::DatabaseError(format!(
            "Article with ID {} not found after update",
            article_id
        ))
    })?;

    let author = repos
        .users
        .find_by_id(&updated_article.author_id)
        .await?
        .map(|user| ArticleAuthor {
            avatar: user.avatar,
            username: user.username,
        });

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Article published successfully",
    );

    Ok(Json(ArticleResponse::from_article(
        updated_article,
        author,
    )?))
}

#[api_operation(
    summary = "Update an article",
    description = "Update an existing article with the provided information",
    tag = "Articles",
    skip
)]
async fn update_article(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    body: Json<ArticleRequest>,
    article_id: Path<String>,
) -> ApiResult<Json<ArticleResponse>> {
    let article_id = article_id.into_inner();

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Updating article",
    );

    let article_request = body.into_inner();

    let mut update = BlogArticleUpdate::new()
        .with_content(&article_request.content)
        .with_description(&article_request.description)
        .with_tags(article_request.tags)
        .with_title(&article_request.title)
        .with_updated_at_to_now();

    if let Some(cover) = article_request.cover {
        update = update.with_cover(&cover);
    }

    let update_result = repos.blog_articles.update(&article_id, update).await?;

    let updated_article = update_result.ok_or_else(|| {
        ApiError::DatabaseError(format!(
            "Article with ID {} not found after update",
            article_id
        ))
    })?;

    let author = repos
        .users
        .find_by_id(&updated_article.author_id)
        .await?
        .map(|user| ArticleAuthor {
            avatar: user.avatar,
            username: user.username,
        });

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Article updated successfully",
    );

    Ok(Json(ArticleResponse::from_article(
        updated_article,
        author,
    )?))
}

#[api_operation(
    summary = "Delete an article",
    description = "Delete an existing article by its ID",
    tag = "Articles",
    skip
)]
async fn delete_article(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
    article_id: Path<String>,
) -> ApiResult<Json<ArticleDeleteResponse>> {
    let article_id = article_id.into_inner();

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Deleting article",
    );

    let delete_result = repos.blog_articles.delete(&article_id).await?;

    if delete_result.deleted_count == 0 {
        return Err(ApiError::NotFound(format!(
            "Article with ID {} not found",
            article_id
        )));
    }

    info!(
        code = %LogCode::Request,
        article_id = %article_id,
        "Article deleted successfully",
    );

    Ok(Json(ArticleDeleteResponse { success: true }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{article_id}").service(
            resource("")
                .route(get().to(get_article))
                .route(post().to(publish_article))
                .route(patch().to(update_article))
                .route(delete().to(delete_article)),
        ),
    );
}
