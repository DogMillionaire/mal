from reader import MalSyntaxError, Reader, read_form, read_str
from printer import pr_str
from mal_token import MalToken
from mal_error import MalError, MalNoInputError
import mal_readline

def READ(source: str) -> MalToken:
    reader = read_str(source)
    return read_form(reader)

def EVAL(form: MalToken) -> MalToken:
    return form

def PRINT(form: MalToken) -> str:
    return pr_str(form, True)

def rep(source: str) -> str:
    return PRINT(EVAL(READ(source)))

def main() -> None:
    while True:
        try:
            
            print(rep(mal_readline.input_('user> ')))
        except MalNoInputError:
            continue
        except MalError as e:
            print(f"Error: {e}")
        except EOFError:
            break

if __name__ == '__main__':
    main()
