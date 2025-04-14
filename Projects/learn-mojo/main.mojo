def main():
    var name: String = input("Enter your name: ") # explicit variable declaration
    greeting = "Hello, " + name + "!" # implicit variable declaration
    # greeting = 5 # This would cause a type error
    print(greeting)

def loop():
    for x in range(5):
        print("Loop-" + x)