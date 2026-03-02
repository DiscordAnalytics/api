mod article;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, post, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::{OptionalAuth, RequireAdmin},
    domain::{
        error::{ApiError, ApiResult},
        models::BlogArticle,
    },
    openapi::schemas::{ArticleRequest, ArticleResponse},
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get all articles",
    description = "Retrieve a list of all articles in the system",
    tag = "Articles"
)]
async fn get_articles(
    auth: OptionalAuth,
    repos: Data<Repositories>,
) -> ApiResult<Json<Vec<ArticleResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all articles",
    );

    let articles = if let Some(ctx) = &auth.0
        && ctx.is_admin()
    {
        repos.blog_articles.find_all().await?
    } else {
        repos.blog_articles.find_all_published().await?
    };

    let article_reponses = articles
        .into_iter()
        .map(|a| {
            ArticleResponse::try_from(a).map(|mut r| {
                r.content = None;
                r
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        "All articles fetched successfully",
    );

    Ok(Json(article_reponses))
}

#[api_operation(
    summary = "Create a new article",
    description = "Create a new article with the provided information",
    tag = "Articles"
)]
async fn create_article(
    auth: RequireAdmin,
    repos: Data<Repositories>,
    body: Json<ArticleRequest>,
) -> ApiResult<Json<ArticleResponse>> {
    info!(
        code = %LogCode::Request,
        "Creating a new article",
    );

    let user_id = auth.0.user_id.as_deref().ok_or(ApiError::Unauthorized)?;

    let article_request = &body.into_inner();

    let mut new_article = BlogArticle::new(
        user_id,
        &article_request.content,
        &article_request.description,
        article_request.tags.clone(),
        &article_request.title,
    )?;

    if let Some(cover) = &article_request.cover {
        new_article = new_article.with_cover(cover);
    }

    repos.blog_articles.insert(&new_article).await?;

    let article_response = ArticleResponse::try_from(new_article)?;

    info!(
        code = %LogCode::Request,
        "Article created successfully",
    );

    Ok(Json(article_response))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/articles")
            .service(
                resource("")
                    .route(get().to(get_articles))
                    .route(post().to(create_article)),
            )
            .configure(article::configure),
    );
}
