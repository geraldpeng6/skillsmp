# skillsmp-search (sks)

SkillsMP 语义搜索 CLI - 搜索 [skillsmp.com](https://skillsmp.com) 上的 AI Skills。

## 安装

```bash
npm install -g skillsmp-search
```

## 使用

```bash
# 搜索技能
sks python

# 指定返回数量
sks rust --limit 5

# 按星标排序
sks "code review" --sort stars

# 分页
sks typescript --page 2
```

## 输出格式

输出 AI 友好的 JSON 格式：

```json
{
  "query": "python",
  "total_results": 969,
  "page": 1,
  "skills": [
    {
      "name": "python",
      "author": "lambdamechanic",
      "description": "...",
      "stars": 0,
      "url": "https://skillsmp.com/skills/..."
    }
  ]
}
```

## 参数

| 参数 | 短选项 | 说明 | 默认值 |
|------|--------|------|--------|
| `--limit` | `-l` | 返回数量 | 10 |
| `--page` | `-p` | 页码 | 1 |
| `--sort` | `-s` | 排序方式 (recent/stars) | recent |

## License

MIT
