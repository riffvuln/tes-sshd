def main():
    """HELLO WORLD, VARIABLES AND INPUT OUTPUT"""
    var name: String = input("Enter your name: ") # explicit variable declaration
    greeting = "Hello, " + name + "!" # implicit variable declaration
    # greeting = 5 # This would cause a type error
    print(greeting)
    """from now only function calling"""
    loop() # Call the loop function
    print_hello()

    """Blocks and Statements in Mojo"""
    # - Blocks: Code sections enclosed by : that define scope
    # - Statements: Individual instructions like:
    #   * Variable declarations/assignments
    #   * Control flow (if, while, for)
    #   * Function calls and returns
def loop():
    for x in range(5):
        if x%2 == 0:
            print(x, "is even")
        else:
            print(x, "is odd")

def print_hello():
    var text: String = String(",")
                        .join("Hello", "World") # Chain function calls accross multiple lines.
    print(text)