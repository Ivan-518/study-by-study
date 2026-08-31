export type LearningTrack = '应用开发' | '模型原理'

export type Lesson = {
  id: string
  track: LearningTrack
  title: string
  description: string
  duration: number
  level: '基础' | '进阶'
  concepts: string[]
  prerequisites: string[]
  overview: string
  keyPoints: string[]
  code: string
  practice: string
  sourceLabel: string
  sourceUrl: string
}

export const lessons: Lesson[] = [
  {
    id: 'rag-retrieval-basics', track: '应用开发', title: 'RAG：从问题到可靠上下文', duration: 20, level: '基础',
    description: '理解检索增强生成如何补足模型的知识边界，并建立可验证的最小检索链路。',
    concepts: ['RAG', 'Embedding', '上下文构建'], prerequisites: ['LLM 基础', '文本切分'],
    overview: 'RAG 不等于“把更多文本塞给模型”。它先把问题转成可检索的表示，再用高相关、可追溯的片段组成上下文，让模型基于证据回答。质量的关键在检索、重排和引用，而不是提示词长度。',
    keyPoints: ['把知识库切成语义完整、可定位的片段。', '召回阶段宁可多取，再用重排减少噪声。', '每个回答都应能回到原始资料验证。'],
    code: "const context = retrieve(query, { topK: 12 })\n  .then(rerank(query))\n  .then(items => items.slice(0, 4))\n\nconst answer = await model.generate({\n  prompt: `仅依据以下证据回答：${context}`\n})",
    practice: '用任意 3 篇技术文档建立一个小知识库。分别比较只召回和“召回 + 重排”时的答案，并记录一个失败案例。',
    sourceLabel: 'LangChain RAG 概念文档', sourceUrl: 'https://python.langchain.com/docs/concepts/rag/',
  },
  {
    id: 'agent-tool-loop', track: '应用开发', title: 'Agent：规划、工具与可观察循环', duration: 25, level: '进阶',
    description: '用受控的决策循环理解 Agent，而不是把它当成一次模型调用。',
    concepts: ['Agent', '工具调用', '可观测性'], prerequisites: ['结构化输出', 'RAG 基础'],
    overview: '一个可靠的 Agent 将目标拆成有限的步骤，在每一步选择工具、检查工具结果，并保留轨迹。工程重点是限制工具权限、定义停止条件、记录中间状态，以及在失败时能降级或交给人处理。',
    keyPoints: ['规划回答“下一步做什么”，工具回答“如何执行”。', '工具输入与输出必须有可验证的结构化 schema。', '轨迹、耗时、成本和失败原因是调试 Agent 的基础。'],
    code: "while (!state.done && state.steps < 6) {\n  const action = await planner.decide(state)\n  const result = await tools[action.name](action.input)\n  trace.record({ action, result })\n  state = reduce(state, result)\n}",
    practice: '为“查询本周 GitHub AI 项目”设计一个最多 3 步的 Agent。写下工具 schema、停止条件和一个失败时的降级策略。',
    sourceLabel: 'OpenAI Agents 指南', sourceUrl: 'https://platform.openai.com/docs/guides/agents',
  },
  {
    id: 'rag-evaluation', track: '应用开发', title: '评估 RAG：定位检索与生成问题', duration: 18, level: '进阶',
    description: '建立可重复的评估集，拆开检索质量与回答质量，而不是凭感觉调参。',
    concepts: ['RAG', '评估', '重排'], prerequisites: ['RAG 基础'],
    overview: 'RAG 的评估应先问“正确证据是否被找回”，再问“回答是否忠于证据”。把两者混在一起会让问题难以定位。一个小而代表性的题集比大量随机样本更适合个人项目迭代。',
    keyPoints: ['检索指标关注命中、覆盖和排序。', '回答指标关注事实一致性、完整性和引用。', '将失败案例分类，才能决定该改切分、召回、重排还是提示词。'],
    code: "const report = evaluate(dataset, {\n  retrieval: ['recall_at_k', 'mrr'],\n  generation: ['groundedness', 'citation_coverage']\n})\nconsole.table(report.byFailureType)",
    practice: '为上一节的小知识库写 5 个问题和标准证据片段，至少包含一个“无法从资料回答”的问题。',
    sourceLabel: 'Ragas 评估概念', sourceUrl: 'https://docs.ragas.io/en/stable/concepts/metrics/',
  },
  {
    id: 'transformer-attention', track: '模型原理', title: 'Attention：模型如何选择上下文', duration: 22, level: '基础',
    description: '用 Query、Key、Value 的直觉解释注意力，并连接到长上下文与检索场景。',
    concepts: ['Transformer', 'Attention', '上下文窗口'], prerequisites: ['向量与矩阵基础'],
    overview: '注意力机制让每个 token 根据当前目标动态读取其他 token 的信息。Query 表示“我要找什么”，Key 表示“我能匹配什么”，Value 是真正读取的内容；相似度经过 softmax 后成为读取权重。',
    keyPoints: ['注意力是内容寻址，不是简单的平均。', '多头注意力让不同子空间关注不同关系。', '上下文越长，计算和注意力分配都更有挑战。'],
    code: "scores = (Q @ K.transpose(-2, -1)) / sqrt(d_k)\nweights = softmax(scores, dim=-1)\noutput = weights @ V",
    practice: '手算一个 2×2 的注意力权重矩阵：先写出两个 Query 与两个 Key 的点积，再解释较大权重代表什么。',
    sourceLabel: 'The Illustrated Transformer', sourceUrl: 'https://jalammar.github.io/illustrated-transformer/',
  },
  {
    id: 'embeddings-retrieval', track: '模型原理', title: 'Embedding：语义检索为什么可行', duration: 16, level: '基础',
    description: '理解文本向量、相似度和检索误差如何直接影响 RAG 的答案。',
    concepts: ['Embedding', '向量检索', 'RAG'], prerequisites: ['余弦相似度'],
    overview: 'Embedding 将文本映射到向量空间，使语义接近的片段更可能相邻。它不理解“真相”，只提供候选相似度；领域术语、切分方式和查询表达都会改变召回结果。',
    keyPoints: ['余弦相似度比较方向，常用于归一化 embedding。', '文档切分决定检索粒度与上下文完整性。', '向量召回需要和关键词、重排结合以降低漏召回。'],
    code: "const scores = documents.map(doc => cosine(queryVector, doc.vector))\nconst candidates = topK(documents, scores, 10)\nreturn rerank(query, candidates)",
    practice: '为“Agent 如何记忆”写出三种不同措辞的查询，思考它们可能召回不同内容的原因。',
    sourceLabel: 'OpenAI Embeddings 指南', sourceUrl: 'https://platform.openai.com/docs/guides/embeddings',
  },
  {
    id: 'inference-quantization', track: '模型原理', title: '推理与量化：部署时的质量—成本权衡', duration: 19, level: '进阶',
    description: '理解上下文长度、KV Cache 与量化如何影响本地和云端推理。',
    concepts: ['推理', '量化', '本地 AI'], prerequisites: ['Transformer', '线性代数基础'],
    overview: '模型部署不是只看参数量。响应延迟受预填充、生成速度、KV Cache 和硬件带宽共同影响；量化通过更低精度存储与计算降低资源消耗，但必须用你的任务验证质量损失。',
    keyPoints: ['首 token 延迟与后续 token 生成速度是两类不同问题。', 'KV Cache 用显存换取多轮或长上下文的速度。', '选择量化等级前，要在目标任务上比较准确性与稳定性。'],
    code: "const profile = {\n  contextTokens: 8_000,\n  targetLatencyMs: 1_500,\n  quantization: 'int4'\n}\nassertQuality(profile, evaluationSet)",
    practice: '为一个本地 RAG 助手设定延迟、显存和质量三项约束，并说明你会优先压缩哪一项及原因。',
    sourceLabel: 'Hugging Face Transformers 性能指南', sourceUrl: 'https://huggingface.co/docs/transformers/perf_infer_gpu_one',
  },
]

export const lessonById = (id: string) => lessons.find((lesson) => lesson.id === id)
