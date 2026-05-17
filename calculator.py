def add(a, b):
    return a + b

def multiply(a, b):
    return a * b

def divide(a, b):
    if b == 0:
        return "Error: Division by zero"
    return a / b

if __name__ == "__main__":
    print(f"Addition: {add(10, 20)}")
    print(f"Multiplication: {multiply(10, 20)}")
    print(f"Division: {divide(10, 2)}")
# Final test comment
def power(a, b): return a ** b
# Log test 2
