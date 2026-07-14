# HTML Parser
A really basic HTML Parser written to learn some basics. I don't actually recommend using it. It barely works.

The parser works in three steps:
   - Read in the file to the lexer. The lexer converts characters and strings into tokens to be further processed.
   - Read the tokens into the token_parser. This step groups the tokens into opening tags, closing tags, and singular tags
   - Read the  tokens into the tree_builder. This creates a tree structure with the tags by looping through the tokens. The process looks something like:
      - Get token
      - If opening, link as a child of current parent, and set as current parent
      - If singular, set as a child of the current parent
      - If closing, set the parent of the current parent to be the current parent (move up (down?) the tree)

# Program Structure

<pre>
src
 └ html_parser
    └ lexer.rs            - First Pass. Converts the raw text into lexemes or "LexerTokens".
    └ mod.rs              - Holds the references to the other modules 
    └ token_parser.rs     - Second Pass. Converts the Lexer Tokens int tags like &lthtml&gt and &lt/html&gt 
    └ tree_builder.rs     - Third Pass. Creates the tree structure for the page.
 └ main.rs
resources
 └ html_test.html         - HTML file that gets read into the parser.
</pre>
