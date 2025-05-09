#include <filesystem>
#include "FileUtils.hpp"
#include "Parser.hpp"

int wmain(int argc, wchar_t** argv)
{
    std::ignore = argc, argv;

    try
    {
        const std::wstring file_path = L"C:\\Projects\\Kod\\Testing\\text.kod";
        const std::wstring buffer = FileUtils::read_whole_file(file_path);
        auto lexer = std::make_unique<Kod::Lexer>(buffer, file_path);
        Kod::LexerState state = lexer->get_state();

        Kod::Token token;
        do
        {
            token = lexer->get_next_token();
            std::wcout << token << std::endl;
        } while (!token.is(Kod::TokenType::END_OF_INPUT));

        lexer->restore_state(state);
        Kod::Parser parser(std::move(lexer));

        Kod::AstNodeUPtr root = parser.parse();
        std::wcout << *root << std::endl;

    }
    catch (const KodException& e)
    {
        std::wcout << L"Caught kod error: " << static_cast<uint32_t>(e.get_status()) << ", " <<
            e.get_additional_info() << std::endl;
    }
    catch (const std::exception& e)
    {
        std::wcout << L"Caught error: " << e.what() << std::endl;
    }
    catch (...)
    {
        std::wcout << L"Caught an unknown error" << std::endl;
    }

    return 0;
}
