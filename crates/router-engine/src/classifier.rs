use serde::{Deserialize, Serialize};

/// 任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// 代码生成
    CodeGeneration,
    /// 代码解释
    CodeExplanation,
    /// 翻译/本地化
    Translation,
    /// 创意写作
    CreativeWriting,
    /// 分析推理/数学
    Analysis,
    /// 摘要/信息提取
    Summarization,
    /// 问答/知识检索
    GeneralQa,
    /// 工具调用/函数调用
    ToolUse,
    /// 头脑风暴
    Brainstorming,
}

impl TaskType {
    /// 所有任务类型列表
    pub fn all() -> Vec<TaskType> {
        vec![
            TaskType::CodeGeneration,
            TaskType::CodeExplanation,
            TaskType::Translation,
            TaskType::CreativeWriting,
            TaskType::Analysis,
            TaskType::Summarization,
            TaskType::GeneralQa,
            TaskType::ToolUse,
            TaskType::Brainstorming,
        ]
    }

    /// 任务类型名称
    pub fn name(&self) -> &str {
        match self {
            TaskType::CodeGeneration => "code_generation",
            TaskType::CodeExplanation => "code_explanation",
            TaskType::Translation => "translation",
            TaskType::CreativeWriting => "creative_writing",
            TaskType::Analysis => "analysis",
            TaskType::Summarization => "summarization",
            TaskType::GeneralQa => "general_qa",
            TaskType::ToolUse => "tool_use",
            TaskType::Brainstorming => "brainstorming",
        }
    }

    /// 中文名称
    pub fn display_name(&self) -> &str {
        match self {
            TaskType::CodeGeneration => "代码生成",
            TaskType::CodeExplanation => "代码解释",
            TaskType::Translation => "翻译/本地化",
            TaskType::CreativeWriting => "创意写作",
            TaskType::Analysis => "分析推理",
            TaskType::Summarization => "摘要提取",
            TaskType::GeneralQa => "问答知识",
            TaskType::ToolUse => "工具调用",
            TaskType::Brainstorming => "头脑风暴",
        }
    }
}

/// 任务分类结果
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub primary_type: TaskType,
    pub confidence: f64,
    pub all_scores: Vec<(TaskType, f64)>,
}

/// 任务分类器
pub struct Classifier {
    rules: Vec<ClassificationRule>,
}

struct ClassificationRule {
    task_type: TaskType,
    keywords: Vec<&'static str>,
    patterns: Vec<&'static str>,
    weight: f64,
}

impl Classifier {
    pub fn new() -> Self {
        let rules = Self::default_rules();
        Self { rules }
    }

    fn default_rules() -> Vec<ClassificationRule> {
        vec![
            ClassificationRule {
                task_type: TaskType::CodeGeneration,
                keywords: vec![
                    "write a function", "implement", "create a program", "write code",
                    "generate code", "编写代码", "写一个函数", "实现一个",
                    "代码生成", "写一个程序", "coding", "programming",
                    "rust code", "python script", "javascript function",
                    "algorithm", "data structure", "sort", "search",
                    "print", "hello world", "def ", "fn ", "function",
                    "class ", "impl ", "trait ",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::CodeExplanation,
                keywords: vec![
                    "explain this code", "what does this code do", "code review",
                    "解释代码", "这段代码", "代码含义", "review this code",
                    "explain the code", "how does this work", "code analysis",
                    "understand this code", "代码解释",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::Translation,
                keywords: vec![
                    "translate", "translation", "翻译", "translate to",
                    "翻译成", "翻訳", "übersetzen", "traduire",
                    "traducir", "translation from", "translate this",
                    "localization", "i18n", "l10n",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::CreativeWriting,
                keywords: vec![
                    "write a story", "write a poem", "creative writing",
                    "写一个故事", "写一首诗", "创作", "小说",
                    "story about", "poem about", "write an essay",
                    "narrative", "fiction", "write a novel",
                    "创意写作", "散文", "剧本", "script",
                    "dialogue", "描写",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::Analysis,
                keywords: vec![
                    "analyze", "analysis", "compare", "contrast",
                    "逻辑推理", "分析", "数学", "math",
                    "solve", "calculate", "computation", "equation",
                    "formula", "证明", "推理", "逻辑",
                    "reasoning", "logical", "deduce", "归纳",
                    "statistics", "probability", "统计",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::Summarization,
                keywords: vec![
                    "summarize", "summary", "summarize this",
                    "总结", "摘要", "归纳", "提炼",
                    "tl;dr", "tldr", "key points", "要点",
                    "brief summary", "short version", "abstract",
                    "总结一下", "概括", "浓缩",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::GeneralQa,
                keywords: vec![
                    "what is", "how to", "why is", "when did",
                    "where is", "who is", "tell me about",
                    "什么是", "怎么", "为什么", "如何",
                    "是什么", "有什么区别", "怎么做",
                    "explain", "definition", "meaning of",
                    "difference between", "what are",
                ],
                patterns: vec![],
                weight: 0.7,
            },
            ClassificationRule {
                task_type: TaskType::ToolUse,
                keywords: vec![
                    "use tool", "call function", "execute command",
                    "run shell", "search web", "look up",
                    "使用工具", "调用函数", "执行命令",
                    "天气", "weather", "search", "查询",
                    "find information", "lookup", "look up",
                ],
                patterns: vec![],
                weight: 1.0,
            },
            ClassificationRule {
                task_type: TaskType::Brainstorming,
                keywords: vec![
                    "brainstorm", "ideas", "suggest", "recommend",
                    "头脑风暴", "想法", "建议", "推荐",
                    "what are some", "give me ideas", "creative ideas",
                    "options for", "ways to", "possible solutions",
                    "你有什么想法", "有什么建议",
                ],
                patterns: vec![],
                weight: 1.0,
            },
        ]
    }

    /// 对请求内容进行分类
    pub fn classify(&self, text: &str) -> ClassificationResult {
        let text_lower = text.to_lowercase();
        let mut scores: Vec<(TaskType, f64)> = TaskType::all()
            .into_iter()
            .map(|t| (t, self.score_task(&text_lower, t)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let primary = scores.first().cloned().unwrap_or((TaskType::GeneralQa, 0.0));

        ClassificationResult {
            primary_type: primary.0,
            confidence: primary.1,
            all_scores: scores,
        }
    }

    fn score_task(&self, text: &str, task_type: TaskType) -> f64 {
        let mut score = 0.0;
        let mut matched = 0;

        for rule in &self.rules {
            if rule.task_type != task_type {
                continue;
            }
            for keyword in &rule.keywords {
                if text.contains(keyword) {
                    score += rule.weight;
                    matched += 1;
                }
            }
        }

        // 长度归一化，短文本匹配更少关键字
        let word_count = text.split_whitespace().count().max(1) as f64;
        let normalized = if matched > 0 {
            (score / word_count * 100.0).min(1.0)
        } else {
            0.0
        };

        normalized
    }
}

impl Default for Classifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code_generation() {
        let classifier = Classifier::new();
        let result = classifier.classify("Write a function to sort an array in Python");
        assert_eq!(result.primary_type, TaskType::CodeGeneration);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_classify_translation() {
        let classifier = Classifier::new();
        let result = classifier.classify("Translate this sentence to Chinese");
        assert_eq!(result.primary_type, TaskType::Translation);
    }

    #[test]
    fn test_classify_summarization() {
        let classifier = Classifier::new();
        let result = classifier.classify("Summarize this article for me");
        assert_eq!(result.primary_type, TaskType::Summarization);
    }

    #[test]
    fn test_classify_analysis() {
        let classifier = Classifier::new();
        let result = classifier.classify("Analyze the pros and cons of this approach");
        assert_eq!(result.primary_type, TaskType::Analysis);
    }

    #[test]
    fn test_classify_creative_writing() {
        let classifier = Classifier::new();
        let result = classifier.classify("Write a story about a brave knight");
        assert_eq!(result.primary_type, TaskType::CreativeWriting);
    }

    #[test]
    fn test_classify_code_explanation() {
        let classifier = Classifier::new();
        // 使用明确的代码解释请求
        let result = classifier.classify("explain this code to me, what does it do");
        assert_eq!(result.primary_type, TaskType::CodeExplanation);
    }

    #[test]
    fn test_classify_default_qa() {
        let classifier = Classifier::new();
        let result = classifier.classify("What is the capital of France?");
        assert_eq!(result.primary_type, TaskType::GeneralQa);
    }

    #[test]
    fn test_classify_confidence_ordering() {
        let classifier = Classifier::new();
        let result = classifier.classify("Write a Python function that sorts numbers");
        // 最高分应该是 code_generation
        let top = result.all_scores.first().unwrap();
        assert_eq!(top.0, TaskType::CodeGeneration);
        // 应该比 general_qa 高
        let qa_score = result.all_scores.iter()
            .find(|(t, _)| *t == TaskType::GeneralQa)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        assert!(top.1 >= qa_score);
    }
}