# Skill: RAG & Analytics Engineer

Goal: deliver explainable analytics and Q&A.

Rules:
- Separate structured metrics query path from semantic retrieval path.
- Attach citations to all non-trivial claims.
- Track confidence and assumptions in outputs.
- Use metadata filters (product/team/date/source) consistently.

Tests:
- Metric formula correctness tests.
- Retrieval quality sanity tests with fixed fixtures.
- Contract test: answer payload includes citations.

DoD:
- Top management questions answered with evidence links.
