//! SkillsMP 语义搜索 CLI 工具
//!
//! 用于搜索 skillsmp.com 上的 AI Skills，输出 JSON 格式便于 AI 读取。
//!
//! # 使用方法
//! ```bash
//! # 搜索 python 相关技能
//! sks python
//!
//! # 指定返回数量和排序
//! sks rust --limit 5 --sort stars
//! ```

// ============================================================================
// 依赖导入
// ============================================================================
// anyhow: 简化错误处理，Context trait 用于给错误添加上下文信息
// clap: 命令行参数解析库，Parser trait 自动生成参数解析代码
// serde: 序列化/反序列化库，用于 JSON 转换
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

// API 基础地址
const BASE_URL: &str = "https://skillsmp.com/api/v1";

// ============================================================================
// 命令行参数定义
// ============================================================================
// #[derive(Parser)] 宏自动为 Cli 结构体生成命令行解析代码
// 类似于 Python 的 argparse 或 click
#[derive(Parser)]
#[command(name = "sks")]
#[command(about = "SkillsMP Semantic Search", long_about = None)]
struct Cli {
    /// 搜索关键词（必填参数）
    query: String,

    /// 返回结果数量
    /// -l 或 --limit，默认 10
    #[arg(short, long, default_value = "10")]
    limit: u32,

    /// 页码
    /// -p 或 --page，默认 1
    #[arg(short, long, default_value = "1")]
    page: u32,

    /// 排序方式: recent(最近) 或 stars(星标数)
    /// -s 或 --sort，默认 recent
    #[arg(short, long, default_value = "recent")]
    sort: String,

    /// API 密钥
    /// 可通过 --api-key 传入，或设置环境变量 SKILLSMP_API_KEY
    #[arg(long, env = "SKILLSMP_API_KEY")]
    api_key: String,
}

// ============================================================================
// API 响应数据结构
// ============================================================================
// 这些结构体用于将 JSON 响应反序列化为 Rust 对象
// Option<T> 表示该字段可能不存在（类似于其他语言的 nullable）

/// API 响应的顶层结构
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    success: Option<bool>,       // 请求是否成功
    data: Option<ResponseData>,  // 成功时的数据
    error: Option<ApiError>,     // 失败时的错误信息
}

/// 错误信息结构
#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
    code: Option<String>,    // 错误代码
    message: Option<String>, // 错误消息
}

/// 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    skills: Vec<Skill>,       // 技能列表（Vec 类似于其他语言的 Array/List）
    pagination: Pagination,   // 分页信息
}

/// 单个技能的完整信息
/// #[serde(rename_all = "camelCase")] 自动将 snake_case 转换为 camelCase
/// 例如: github_url <-> githubUrl
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Skill {
    id: String,
    name: String,
    author: String,
    description: Option<String>,  // 可选字段，可能为 null
    github_url: Option<String>,
    skill_url: Option<String>,
    stars: Option<u32>,           // u32 = 无符号32位整数
}

/// 分页信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagination {
    page: u32,
    limit: u32,
    total: u32,
    total_pages: u32,
    has_next: bool,
    has_prev: bool,
}

// ============================================================================
// AI 友好的输出结构
// ============================================================================
// 这些结构体用于生成简化的 JSON 输出，只包含 AI 需要的核心信息

/// AI 输出的顶层结构
#[derive(Debug, Serialize)]
struct AiOutput {
    query: String,        // 搜索词
    total_results: u32,   // 总结果数
    page: u32,            // 当前页
    skills: Vec<AiSkill>, // 简化的技能列表
}

/// 简化的技能信息（只保留 AI 需要的字段）
#[derive(Debug, Serialize)]
struct AiSkill {
    name: String,
    author: String,
    description: String,
    stars: u32,
    url: String,
}

// ============================================================================
// 核心功能函数
// ============================================================================

/// 调用 SkillsMP API 搜索技能
///
/// # 参数
/// - `cli`: 命令行参数
///
/// # 返回
/// - `Result<ApiResponse>`: 成功返回 API 响应，失败返回错误
///   Result 是 Rust 的错误处理方式，类似于 try/catch
fn search_skills(cli: &Cli) -> Result<ApiResponse> {
    // 构建请求 URL
    // format! 宏类似于 f-string 或 string.format()
    let url = format!(
        "{}/skills/search?q={}&limit={}&page={}&sortBy={}",
        BASE_URL,
        urlencoding::encode(&cli.query), // URL 编码搜索词
        cli.limit,
        cli.page,
        cli.sort
    );

    // 创建 HTTP 客户端并发送 GET 请求
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", cli.api_key))
        .header("Content-Type", "application/json")
        .send()
        .context("发送请求失败")?; // ? 操作符：出错时提前返回错误

    // 将响应 JSON 反序列化为 ApiResponse 结构体
    response.json().context("解析响应失败")
}

/// 程序入口点
///
/// # 返回
/// - `Result<()>`: () 表示空值（类似于 void），Result 包装表示可能出错
fn main() -> Result<()> {
    // 加载 .env 文件中的环境变量（如果存在）
    // .ok() 忽略加载失败的错误（文件不存在时）
    dotenvy::dotenv().ok();

    // 解析命令行参数
    let cli = Cli::parse();

    // 调用 API
    let response = search_skills(&cli)?;

    // 检查 API 是否返回错误
    // if let 是 Rust 的模式匹配，这里检查 success 是否为 Some(false)
    if let Some(false) = response.success {
        if let Some(err) = response.error {
            // eprintln! 输出到 stderr
            eprintln!(
                "Error: {}",
                err.message.unwrap_or_else(|| "Unknown error".to_string())
            );
            std::process::exit(1);
        }
    }

    // 提取数据，如果不存在则返回错误
    let data = response.data.context("响应中没有数据")?;

    // 构建 AI 友好的输出结构
    // into_iter() 将 Vec 转换为迭代器
    // map() 对每个元素进行转换
    // collect() 将迭代器收集回 Vec
    let output = AiOutput {
        query: cli.query,
        total_results: data.pagination.total,
        page: data.pagination.page,
        skills: data
            .skills
            .into_iter()
            .map(|s| AiSkill {
                name: s.name,
                author: s.author,
                // unwrap_or_default() 如果是 None 则返回类型的默认值（空字符串/0）
                description: s.description.unwrap_or_default(),
                stars: s.stars.unwrap_or(0),
                url: s.skill_url.unwrap_or_default(),
            })
            .collect(),
    };

    // 输出格式化的 JSON
    println!("{}", serde_json::to_string_pretty(&output)?);

    // 返回成功（空的 Ok）
    Ok(())
}
