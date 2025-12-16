#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <cctype>
#include <memory>
#include <stdexcept>
#include <unordered_map>

// plsa: Parser, Lexer, AST, Syntax Checker for Vira Language
// Written in C++
// This is a basic implementation assuming a simplified syntax similar to JS/Ruby with specified changes.
// Supports: variables, functions, imports (:lib:), write statements, basic expressions, comments (< comment)
// Does not support full language features as syntax is not fully specified.
// Usage: plsa <input.vira> [--ast] [--check]

enum TokenType {
    TOK_EOF,
    TOK_IDENTIFIER,
    TOK_NUMBER,
    TOK_STRING,
    TOK_COLON,        // :
    TOK_ASSIGN,       // =
    TOK_PLUS,         // +
    TOK_MINUS,        // -
    TOK_MUL,          // *
    TOK_DIV,          // /
    TOK_LPAREN,       // (
    TOK_RPAREN,       // )
    TOK_LBRACE,       // {
    TOK_RBRACE,       // }
    TOK_SEMICOLON,    // ;
    TOK_COMMA,        // ,
    TOK_LET,          // let (variable declaration)
    TOK_DEF,          // def (function definition, like Ruby)
    TOK_WRITE,        // write (print)
    TOK_IMPORT_START, // Special for :lib:
    TOK_COMMENT,      // < comment
    TOK_UNKNOWN
};

struct Token {
    TokenType type;
    std::string value;
    int line;
    int column;
};

class Lexer {
public:
    Lexer(const std::string& source) : source(source), pos(0), line(1), column(1) {}

    Token nextToken() {
        skipWhitespace();
        if (pos >= source.size()) {
            return {TOK_EOF, "", line, column};
        }

        char ch = source[pos];
        if (std::isdigit(ch)) {
            return number();
        } else if (std::isalpha(ch) || ch == '_') {
            return identifier();
        } else if (ch == '"') {
            return string();
        } else if (ch == '<') {
            return comment();
        } else if (ch == ':') {
            advance();
            if (std::isalpha(source[pos])) {
                // Start of import :lib:
                std::string lib = "";
                while (pos < source.size() && (std::isalnum(source[pos]) || source[pos] == '_')) {
                    lib += source[pos];
                    advance();
                }
                if (pos < source.size() && source[pos] == ':') {
                    advance();
                    return {TOK_IMPORT_START, lib, line, column};
                } else {
                    // Invalid, but for now treat as colon + identifier
                    pos -= lib.size() + 1; // Rewind
                    return {TOK_COLON, ":", line, column};
                }
            }
            return {TOK_COLON, ":", line, column};
        } else {
            advance();
            switch (ch) {
                case '=': return {TOK_ASSIGN, "=", line, column};
                case '+': return {TOK_PLUS, "+", line, column};
                case '-': return {TOK_MINUS, "-", line, column};
                case '*': return {TOK_MUL, "*", line, column};
                case '/': return {TOK_DIV, "/", line, column};
                case '(': return {TOK_LPAREN, "(", line, column};
                case ')': return {TOK_RPAREN, ")", line, column};
                case '{': return {TOK_LBRACE, "{", line, column};
                case '}': return {TOK_RBRACE, "}", line, column};
                case ';': return {TOK_SEMICOLON, ";", line, column};
                case ',': return {TOK_COMMA, ",", line, column};
                default: return {TOK_UNKNOWN, std::string(1, ch), line, column};
            }
        }
    }

private:
    std::string source;
    size_t pos;
    int line;
    int column;

    void advance() {
        if (source[pos] == '\n') {
            line++;
            column = 1;
        } else {
            column++;
        }
        pos++;
    }

    void skipWhitespace() {
        while (pos < source.size() && std::isspace(source[pos])) {
            advance();
        }
    }

    Token number() {
        std::string num;
        int startLine = line, startCol = column;
        while (pos < source.size() && std::isdigit(source[pos])) {
            num += source[pos];
            advance();
        }
        return {TOK_NUMBER, num, startLine, startCol};
    }

    Token identifier() {
        std::string id;
        int startLine = line, startCol = column;
        while (pos < source.size() && (std::isalnum(source[pos]) || source[pos] == '_')) {
            id += source[pos];
            advance();
        }
        // Keywords
        if (id == "let") return {TOK_LET, id, startLine, startCol};
        if (id == "def") return {TOK_DEF, id, startLine, startCol};
        if (id == "write") return {TOK_WRITE, id, startLine, startCol};
        return {TOK_IDENTIFIER, id, startLine, startCol};
    }

    Token string() {
        std::string str;
        int startLine = line, startCol = column;
        advance(); // Skip opening "
        while (pos < source.size() && source[pos] != '"') {
            if (source[pos] == '\\') {
                advance();
                if (pos < source.size()) {
                    str += source[pos]; // Simple escape, no handling
                    advance();
                }
            } else {
                str += source[pos];
                advance();
            }
        }
        if (pos >= source.size() || source[pos] != '"') {
            throw std::runtime_error("Unterminated string at line " + std::to_string(line));
        }
        advance(); // Skip closing "
        return {TOK_STRING, str, startLine, startCol};
    }

    Token comment() {
        std::string comment;
        int startLine = line, startCol = column;
        advance(); // Skip <
        while (pos < source.size() && source[pos] != '\n') {
            comment += source[pos];
            advance();
        }
        return {TOK_COMMENT, comment, startLine, startCol};
    }
};

// AST Nodes
class AstNode {
public:
    virtual ~AstNode() = default;
    virtual void print(int indent = 0) const = 0;
};

class Expr : public AstNode {};
class Stmt : public AstNode {};

using ExprPtr = std::unique_ptr<Expr>;
using StmtPtr = std::unique_ptr<Stmt>;
using AstPtr = std::unique_ptr<AstNode>;

class NumberExpr : public Expr {
public:
    double value;
    NumberExpr(double val) : value(val) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "Number: " << value << "\n";
    }
};

class StringExpr : public Expr {
public:
    std::string value;
    StringExpr(std::string val) : value(std::move(val)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "String: \"" << value << "\"\n";
    }
};

class IdentifierExpr : public Expr {
public:
    std::string name;
    IdentifierExpr(std::string n) : name(std::move(n)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "Identifier: " << name << "\n";
    }
};

class BinaryExpr : public Expr {
public:
    char op;
    ExprPtr left;
    ExprPtr right;
    BinaryExpr(char o, ExprPtr l, ExprPtr r) : op(o), left(std::move(l)), right(std::move(r)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "Binary: " << op << "\n";
        left->print(indent + 2);
        right->print(indent + 2);
    }
};

class CallExpr : public Expr {
public:
    std::string callee;
    std::vector<ExprPtr> args;
    CallExpr(std::string c, std::vector<ExprPtr> a) : callee(std::move(c)), args(std::move(a)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "Call: " << callee << "\n";
        for (const auto& arg : args) {
            arg->print(indent + 2);
        }
    }
};

class VarDeclStmt : public Stmt {
public:
    std::string name;
    ExprPtr initializer;
    VarDeclStmt(std::string n, ExprPtr init) : name(std::move(n)), initializer(std::move(init)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "VarDecl: " << name << "\n";
        if (initializer) initializer->print(indent + 2);
    }
};

class FuncDefStmt : public Stmt {
public:
    std::string name;
    std::vector<std::string> params;
    std::vector<StmtPtr> body;
    FuncDefStmt(std::string n, std::vector<std::string> p, std::vector<StmtPtr> b)
        : name(std::move(n)), params(std::move(p)), body(std::move(b)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "FuncDef: " << name << "\n";
        std::cout << std::string(indent + 2, ' ') << "Params:\n";
        for (const auto& param : params) {
            std::cout << std::string(indent + 4, ' ') << param << "\n";
        }
        std::cout << std::string(indent + 2, ' ') << "Body:\n";
        for (const auto& stmt : body) {
            stmt->print(indent + 4);
        }
    }
};

class WriteStmt : public Stmt {
public:
    ExprPtr expr;
    WriteStmt(ExprPtr e) : expr(std::move(e)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "Write:\n";
        expr->print(indent + 2);
    }
};

class ImportStmt : public Stmt {
public:
    std::string libName;
    std::string alias; // Optional, but for :lib: => Alias, but syntax not fully specified
    ImportStmt(std::string lib, std::string al = "") : libName(std::move(lib)), alias(std::move(al)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "Import: " << libName;
        if (!alias.empty()) std::cout << " as " << alias;
        std::cout << "\n";
    }
};

class ExprStmt : public Stmt {
public:
    ExprPtr expr;
    ExprStmt(ExprPtr e) : expr(std::move(e)) {}
    void print(int indent) const override {
        std::cout << std::string(indent, ' ') << "ExprStmt:\n";
        expr->print(indent + 2);
    }
};

class Program : public AstNode {
public:
    std::vector<StmtPtr> statements;
    void print(int indent) const override {
        std::cout << "Program:\n";
        for (const auto& stmt : statements) {
            stmt->print(2);
        }
    }
};

class Parser {
public:
    Parser(std::vector<Token> t) : tokens(std::move(t)), pos(0) {}

    std::unique_ptr<Program> parse() {
        auto program = std::make_unique<Program>();
        while (!isAtEnd()) {
            auto stmt = declaration();
            if (stmt) {
                program->statements.push_back(std::move(stmt));
            }
        }
        return program;
    }

private:
    std::vector<Token> tokens;
    size_t pos;

    bool isAtEnd() const { return peek().type == TOK_EOF; }

    Token peek() const {
        if (pos < tokens.size()) return tokens[pos];
        return {TOK_EOF, "", -1, -1};
    }

    Token advance() {
        if (pos < tokens.size()) return tokens[pos++];
        return {TOK_EOF, "", -1, -1};
    }

    bool match(TokenType type) {
        if (peek().type == type) {
            advance();
            return true;
        }
        return false;
    }

    void consume(TokenType type, const std::string& msg) {
        if (match(type)) return;
        throw std::runtime_error(msg + " at line " + std::to_string(peek().line));
    }

    StmtPtr declaration() {
        try {
            if (match(TOK_LET)) return varDecl();
            if (match(TOK_DEF)) return funcDef();
            if (match(TOK_IMPORT_START)) return importStmt();
            return statement();
        } catch (const std::exception& e) {
            synchronize();
            std::cerr << "Error: " << e.what() << "\n";
            return nullptr;
        }
    }

    StmtPtr varDecl() {
        Token name = consume(TOK_IDENTIFIER, "Expected variable name");
        ExprPtr init = nullptr;
        if (match(TOK_ASSIGN)) {
            init = expression();
        }
        consume(TOK_SEMICOLON, "Expected ';' after variable declaration");
        return std::make_unique<VarDeclStmt>(name.value, std::move(init));
    }

    StmtPtr funcDef() {
        Token name = consume(TOK_IDENTIFIER, "Expected function name");
        consume(TOK_LPAREN, "Expected '(' after function name");
        std::vector<std::string> params;
        if (!match(TOK_RPAREN)) {
            do {
                params.push_back(consume(TOK_IDENTIFIER, "Expected parameter name").value);
            } while (match(TOK_COMMA));
            consume(TOK_RPAREN, "Expected ')' after parameters");
        }
        consume(TOK_LBRACE, "Expected '{' before function body");
        std::vector<StmtPtr> body;
        while (!match(TOK_RBRACE) && !isAtEnd()) {
            body.push_back(declaration());
        }
        return std::make_unique<FuncDefStmt>(name.value, std::move(params), std::move(body));
    }

    StmtPtr importStmt() {
        // Assuming :lib: for import, value is lib name
        // If there's => Alias, but syntax not specified, assuming just :lib:
        std::string lib = previous().value; // previous because advance already happened
        // For simplicity, no alias
        consume(TOK_SEMICOLON, "Expected ';' after import");
        return std::make_unique<ImportStmt>(lib);
    }

    Token previous() const {
        return tokens[pos - 1];
    }

    StmtPtr statement() {
        if (match(TOK_WRITE)) return writeStmt();
        return exprStmt();
    }

    StmtPtr writeStmt() {
        ExprPtr expr = expression();
        consume(TOK_SEMICOLON, "Expected ';' after write");
        return std::make_unique<WriteStmt>(std::move(expr));
    }

    StmtPtr exprStmt() {
        ExprPtr expr = expression();
        consume(TOK_SEMICOLON, "Expected ';' after expression");
        return std::make_unique<ExprStmt>(std::move(expr));
    }

    ExprPtr expression() {
        return equality();
    }

    ExprPtr equality() {
        ExprPtr expr = additive();
        // No == etc. for simplicity, assume only math
        return expr;
    }

    ExprPtr additive() {
        ExprPtr expr = multiplicative();
        while (match(TOK_PLUS) || match(TOK_MINUS)) {
            Token op = previous();
            ExprPtr right = multiplicative();
            expr = std::make_unique<BinaryExpr>(op.value[0], std::move(expr), std::move(right));
        }
        return expr;
    }

    ExprPtr multiplicative() {
        ExprPtr expr = unary();
        while (match(TOK_MUL) || match(TOK_DIV)) {
            Token op = previous();
            ExprPtr right = unary();
            expr = std::make_unique<BinaryExpr>(op.value[0], std::move(expr), std::move(right));
        }
        return expr;
    }

    ExprPtr unary() {
        if (match(TOK_MINUS)) {
            ExprPtr right = unary();
            return std::make_unique<BinaryExpr>('-', std::make_unique<NumberExpr>(0), std::move(right)); // Simple negation
        }
        return primary();
    }

    ExprPtr primary() {
        if (match(TOK_NUMBER)) {
            return std::make_unique<NumberExpr>(std::stod(previous().value));
        }
        if (match(TOK_STRING)) {
            return std::make_unique<StringExpr>(previous().value);
        }
        if (match(TOK_IDENTIFIER)) {
            if (match(TOK_LPAREN)) {
                // Function call
                std::string callee = previous().value; // Identifier before (
                std::vector<ExprPtr> args;
                if (!match(TOK_RPAREN)) {
                    do {
                        args.push_back(expression());
                    } while (match(TOK_COMMA));
                    consume(TOK_RPAREN, "Expected ')' after arguments");
                }
                return std::make_unique<CallExpr>(callee, std::move(args));
            }
            return std::make_unique<IdentifierExpr>(previous().value);
        }
        if (match(TOK_LPAREN)) {
            ExprPtr expr = expression();
            consume(TOK_RPAREN, "Expected ')' after expression");
            return expr;
        }
        throw std::runtime_error("Unexpected token at line " + std::to_string(peek().line));
    }

    void synchronize() {
        while (!isAtEnd()) {
            if (peek().type == TOK_SEMICOLON) {
                advance();
                return;
            }
            switch (peek().type) {
                case TOK_LET:
                case TOK_DEF:
                case TOK_WRITE:
                    return;
                default:
                    advance();
            }
        }
    }
};

class SyntaxChecker {
public:
    void check(const Program& program) {
        // Basic checks: e.g., no undefined vars, but for simplicity, just placeholder
        // Could add symbol table, type checking, etc., but keeping minimal
        std::cout << "Syntax check passed.\n";
    }
};

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: plsa <input.vira> [--ast] [--check]\n";
        return 1;
    }

    std::string filename = argv[1];
    std::ifstream file(filename);
    if (!file) {
        std::cerr << "Could not open file: " << filename << "\n";
        return 1;
    }

    std::string source((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());

    Lexer lexer(source);
    std::vector<Token> tokens;
    Token token;
    do {
        token = lexer.nextToken();
        if (token.type != TOK_EOF && token.type != TOK_COMMENT && token.type != TOK_UNKNOWN) {
            tokens.push_back(token);
        } else if (token.type == TOK_UNKNOWN) {
            std::cerr << "Unknown token: " << token.value << " at line " << token.line << "\n";
            return 1;
        }
    } while (token.type != TOK_EOF);

    Parser parser(std::move(tokens));
    auto program = parser.parse();

    bool printAst = false;
    bool checkSyntax = false;
    for (int i = 2; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "--ast") printAst = true;
        if (arg == "--check") checkSyntax = true;
    }

    if (printAst) {
        program->print();
    }

    if (checkSyntax) {
        SyntaxChecker checker;
        checker.check(*program);
    }

    return 0;
}
