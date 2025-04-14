def main():
    var name: String = input("Enter your name: ") # explicit variable declaration
    greeting = "Hello, " + name + "!" # implicit variable declaration
    # greeting = 5 # This would cause a type error
    print(greeting)
    loop() # Call the loop function

# Function to demonstrate a simple loop
# and conditional statements
# This function prints whether numbers from 0 to 4 are even or odd
def loop():
    for x in range(5):
        if x%2 == 0:
            print(x, "is even")
        else:
            print(x, "is odd")