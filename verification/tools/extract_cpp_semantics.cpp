// Copyright 2026 Destiny-rs contributors. MIT.
// A deliberately restricted LibTooling extractor; unknown nodes are errors.
#include <clang/AST/RecursiveASTVisitor.h>
#include <clang/Frontend/FrontendActions.h>
#if __has_include(<clang/UnifiedSymbolResolution/USRGeneration.h>)
#include <clang/UnifiedSymbolResolution/USRGeneration.h>
#else
#include <clang/Index/USRGeneration.h>
#endif
#include <clang/Tooling/CommonOptionsParser.h>
#include <clang/Tooling/Tooling.h>
#include <llvm/Support/JSON.h>
#include <llvm/Support/raw_ostream.h>

using namespace clang;
using namespace clang::tooling;
static llvm::cl::OptionCategory Category("Destiny scalar semantics");

llvm::json::Value expression(const Expr *input) {
    const Expr *e = input->IgnoreParenImpCasts();
    const auto type = e->getType().getUnqualifiedType().getAsString();
    if (type != "double") throw std::runtime_error("unsupported machine type: " + type);
    if (const auto *member = dyn_cast<MemberExpr>(e)) {
        if (!isa<CXXThisExpr>(member->getBase()->IgnoreParenImpCasts()))
            throw std::runtime_error("unsupported member receiver");
        return llvm::json::Object{{"op", "field"}, {"name", member->getMemberDecl()->getNameAsString()}, {"machine_type", type}};
    }
    if (const auto *binary = dyn_cast<BinaryOperator>(e))
        return llvm::json::Object{{"op", "binary"}, {"operator", binary->getOpcodeStr().str()}, {"machine_type", type}, {"left", expression(binary->getLHS())}, {"right", expression(binary->getRHS())}};
    if (const auto *call = dyn_cast<CallExpr>(e)) {
        auto *callee = call->getDirectCallee();
        if (!callee || callee->getNameAsString() != "sqrt" || call->getNumArgs() != 1)
            throw std::runtime_error("unsupported call");
        return llvm::json::Object{{"op", "sqrt"}, {"machine_type", type}, {"value", expression(call->getArg(0))}};
    }
    throw std::runtime_error("unsupported expression: " + std::string(e->getStmtClassName()));
}

class Visitor : public RecursiveASTVisitor<Visitor> {
public:
    llvm::json::Array records;
    bool VisitCXXMethodDecl(CXXMethodDecl *method) {
        const std::string symbol = method->getQualifiedNameAsString();
        if (symbol != "Vector3d::Length" && symbol != "Vector3d::LengthSq") return true;
        if (!method->hasBody()) return true;
        const auto *body = dyn_cast<CompoundStmt>(method->getBody());
        if (!body || body->size() != 1 || method->getNumParams() != 0 || !method->isConst())
            throw std::runtime_error("unreviewed method signature/body");
        const auto *result = dyn_cast<ReturnStmt>(*body->body_begin());
        if (!result || !result->getRetValue()) throw std::runtime_error("expected scalar return");
        llvm::SmallString<128> usr;
        if (index::generateUSRForDecl(method, usr)) throw std::runtime_error("unresolved compiler identity");
        const auto &sm = method->getASTContext().getSourceManager();
        const auto start = sm.getPresumedLoc(method->getBeginLoc());
        records.push_back(llvm::json::Object{
            {"original_symbol", symbol}, {"compiler_usr", usr.str().str()},
            {"signature", method->getType().getAsString()}, {"source_line", int64_t(start.getLine())},
            {"name", symbol == "Vector3d::Length" ? "vector_length" : "vector_length_squared"},
            {"return_type", "double"}, {"expression", expression(result->getRetValue())}});
        return true;
    }
};

class Consumer : public ASTConsumer {
    void HandleTranslationUnit(ASTContext &ctx) override {
        if (ctx.getDiagnostics().hasErrorOccurred()) throw std::runtime_error("translation unit contains errors");
        Visitor visitor;
        visitor.TraverseDecl(ctx.getTranslationUnitDecl());
        if (visitor.records.size() != 2) throw std::runtime_error("missing or repeated source method");
        llvm::outs() << llvm::formatv("{0:2}\n", llvm::json::Value(std::move(visitor.records)));
    }
};

class Action : public ASTFrontendAction {
    std::unique_ptr<ASTConsumer> CreateASTConsumer(CompilerInstance &, llvm::StringRef) override {
        return std::make_unique<Consumer>();
    }
};

int main(int argc, const char **argv) {
    auto options = CommonOptionsParser::create(argc, argv, Category);
    if (!options) { llvm::errs() << options.takeError(); return 2; }
    ClangTool tool(options->getCompilations(), options->getSourcePathList());
    return tool.run(newFrontendActionFactory<Action>().get());
}
