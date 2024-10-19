#include "FileUtils.hpp"
#include "Parser.hpp"

int wmain(int argc, char** argv)
{
    (void)argc;
    (void)argv;

    try
    {
        const std::wstring file_path = L"text.txt";
        const std::wstring buffer = FileUtils::read_whole_file(file_path);
        
        Kod::Parser parser(std::make_unique<Kod::Lexer>(buffer, file_path));

        Kod::AstNodeUPtr root = parser.parse();
        std::wcout << *root << std::endl;

        //Kod::Token token;
        //do
        //{
        //    token = lexer.get_next_token();
        //    std::wcout << token << std::endl;
        //} 
        //while (!token.is(Kod::TokenType::END_OF_INPUT));
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
