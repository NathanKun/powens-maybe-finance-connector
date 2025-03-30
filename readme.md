# Powens Maybe-Finance Connector

(Actually this is not a connector, since maybe has no API for now)

Use Powens' accounts & transactions APIs to retrieve account transactions data.

Convert data to CSV format to be imported manually in Maybe (no maybe API for now).

Use GenAI to guess transaction category, and return it in the CSV. Available transaction categories and examples for helping 
AI are defined in `./ai-prompts`.

Retrieved account, transaction, and AI guessing data are persisted in `./db` in JSON format.