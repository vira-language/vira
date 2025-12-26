#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <map>
#include <cctype>
#include <stdexcept>

enum class TokenType {
    Identifier,
    Keyword,
    Number,
    StringLiteral,
    Punctuator,
    EOFToken
};

struct Token {
    TokenType type;
    std::string value;
    size_t line;
    size_t column;
};

class Lexer {
private:
    std::string input;
    size_t position;
    size_t line;
    size_t column;

public:
    Lexer(const std::string& src) : input(src), position(0), line(1), column(1) {}

    Token nextToken() {
        skipWhitespace();
        if (position >= input.size()) {
            return {TokenType::EOFToken, "", line, column};
        }

        char ch = currentChar();
        if (isalpha(ch) || ch == '_') {
            return lexIdentifierOrKeyword();
        } else if (isdigit(ch)) {
            return lexNumber();
        } else if (ch == '"') {
            return lexString();
        } else if (std::string("+-*/=();{}[]<>,&|!").find(ch) != std::string::npos) {
            advance();
            return {TokenType::Punctuator, std::string(1, ch), line, column - 1};
        } else {
            throw std::runtime_error("Unexpected character: " + std::string(1, ch));
        }
    }

private:
    char currentChar() const {
        return input[position];
    }

    void advance() {
        if (currentChar() == '\n') {
            line++;
            column = 1;
        } else {
            column++;
        }
        position++;
    }

    void skipWhitespace() {
        while (position < input.size() && isspace(currentChar())) {
            advance();
        }
    }

    Token lexIdentifierOrKeyword() {
        std::string id;
        size_t start_col = column;
        while (position < input.size() && (isalnum(currentChar()) || currentChar() == '_')) {
            id += currentChar();
            advance();
        }
        TokenType type = (id == "int" || id == "return" || id == "if" || id == "else" || id == "while" || id == "for")
                         ? TokenType::Keyword : TokenType::Identifier;
        return {type, id, line, start_col};
    }

    Token lexNumber() {
        std::string num;
        size_t start_col = column;
        while (position < input.size() && isdigit(currentChar())) {
            num += currentChar();
            advance();
        }
        return {TokenType::Number, num, line, start_col};
    }

    Token lexString() {
        advance(); // skip opening "
        std::string s;
        size_t start_col = column;
        while (position < input.size() && currentChar() != '"') {
            s += currentChar();
            advance();
        }
        advance(); // skip closing "
        return {TokenType::StringLiteral, s, line, start_col};
    }
};

enum class ASTType {
    Program,
    Function,
    ReturnStmt,
    BinaryOp,
    NumberLiteral,
    Identifier
};

struct ASTNode {
    ASTType type;
    std::string value; // for identifiers, operators, etc.
    std::vector<ASTNode*> children;
    ~ASTNode() {
        for (auto child : children) {
            delete child;
        }
    }
};

class Parser {
private:
    Lexer lexer;
    Token currentToken;

    void eat(TokenType expectedType, const std::string& expectedValue = "") {
        if (currentToken.type == expectedType &&
            (expectedValue.empty() || currentToken.value == expectedValue)) {
            currentToken = lexer.nextToken();
        } else {
            throw std::runtime_error("Syntax error at line " + std::to_string(currentToken.line) +
                                     ", column " + std::to_string(currentToken.column));
        }
    }

    ASTNode* parsePrimary() {
        if (currentToken.type == TokenType::Number) {
            ASTNode* node = new ASTNode{ASTType::NumberLiteral, currentToken.value};
            eat(TokenType::Number);
            return node;
        } else if (currentToken.type == TokenType::Identifier) {
            ASTNode* node = new ASTNode{ASTType::Identifier, currentToken.value};
            eat(TokenType::Identifier);
            return node;
        } else {
            throw std::runtime_error("Unexpected token in primary");
        }
    }

    ASTNode* parseExpr() {
        ASTNode* node = parsePrimary();
        while (currentToken.type == TokenType::Punctuator &&
               (currentToken.value == "+" || currentToken.value == "-" ||
                currentToken.value == "*" || currentToken.value == "/")) {
            std::string op = currentToken.value;
            eat(TokenType::Punctuator, op);
            ASTNode* right = parsePrimary();
            ASTNode* newNode = new ASTNode{ASTType::BinaryOp, op};
            newNode->children.push_back(node);
            newNode->children.push_back(right);
            node = newNode;
        }
        return node;
    }

    ASTNode* parseStatement() {
        if (currentToken.type == TokenType::Keyword && currentToken.value == "return") {
            eat(TokenType::Keyword, "return");
            ASTNode* expr = parseExpr();
            eat(TokenType::Punctuator, ";");
            ASTNode* node = new ASTNode{ASTType::ReturnStmt, ""};
            node->children.push_back(expr);
            return node;
        } else {
            throw std::runtime_error("Unsupported statement");
        }
    }

    ASTNode* parseFunction() {
        eat(TokenType::Keyword, "int");
        std::string name = currentToken.value;
        eat(TokenType::Identifier);
        eat(TokenType::Punctuator, "(");
        eat(TokenType::Punctuator, ")");
        eat(TokenType::Punctuator, "{");
        ASTNode* node = new ASTNode{ASTType::Function, name};
        while (currentToken.type != TokenType::Punctuator || currentToken.value != "}") {
            node->children.push_back(parseStatement());
        }
        eat(TokenType::Punctuator, "}");
        return node;
    }

public:
    Parser(const std::string& src) : lexer(src), currentToken(lexer.nextToken()) {}

    ASTNode* parse() {
        ASTNode* program = new ASTNode{ASTType::Program, ""};
        while (currentToken.type != TokenType::EOFToken) {
            program->children.push_back(parseFunction());
        }
        return program;
    }
};

class SemanticChecker {
private:
    std::map<std::string, std::string> symbolTable; // Simple type table

    void checkExpr(ASTNode* node) {
        if (node->type == ASTType::NumberLiteral) {
            // OK
        } else if (node->type == ASTType::Identifier) {
            if (symbolTable.find(node->value) == symbolTable.end()) {
                throw std::runtime_error("Undefined identifier: " + node->value);
            }
        } else if (node->type == ASTType::BinaryOp) {
            if (node->children.size() != 2) {
                throw std::runtime_error("Binary op needs two children");
            }
            checkExpr(node->children[0]);
            checkExpr(node->children[1]);
            // Type checking could be added here
        } else {
            throw std::runtime_error("Unsupported expr in semantic check");
        }
    }

    void checkStatement(ASTNode* node) {
        if (node->type == ASTType::ReturnStmt) {
            if (node->children.empty()) {
                throw std::runtime_error("Return statement missing expression");
            }
            checkExpr(node->children[0]);
        } else {
            throw std::runtime_error("Unsupported statement in semantic check");
        }
    }

    void checkFunction(ASTNode* node) {
        if (node->type != ASTType::Function) {
            throw std::runtime_error("Expected function");
        }
        // Add function to symbols if needed
        for (auto child : node->children) {
            checkStatement(child);
        }
    }

public:
    void check(ASTNode* program) {
        if (program->type != ASTType::Program) {
            throw std::runtime_error("Expected program");
        }
        for (auto func : program->children) {
            checkFunction(func);
        }
    }
};

int main(int argc, char* argv[]) {
    if (argc != 2) {
        std::cerr << "Usage: plsa <input.vira>" << std::endl;
        return 1;
    }

    std::ifstream file(argv[1]);
    if (!file) {
        std::cerr << "Could not open file: " << argv[1] << std::endl;
        return 1;
    }

    std::string input((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());

    try {
        Parser parser(input);
        ASTNode* ast = parser.parse();

        // Syntax check is implicit in parsing

        SemanticChecker checker;
        checker.check(ast);

        std::cout << "Parsing and checking successful." << std::endl;

        delete ast;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}
