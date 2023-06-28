local function generateNumber()
    return math.random(1, 9999)
end

return {
    zero = 0,
    one = 1,
    number1 = generateNumber(),
    number2 = generateNumber(),
    number3 = generateNumber(),
}
