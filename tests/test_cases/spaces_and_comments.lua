var , var2 -- var2
= true  --
    , --[[]]false

 i --[[]] -= 1
do --[[]]
end
function foo -- foo
    . var -- var
    (param1 --[[string]], p2 , ... --[[rest]] ) -- end of parameters
end
function foo:var -- var method
    (param1 --[[string]], p2 , ... --[[rest]] ) -- end of parameters
    return 0x01 --[[first]] - 0b1 -- number
end

for key --[[comment]], value   in pairs( variable ) do continue end

if --[[condition]] "value" then
elseif --[[other condition]] 'value2'  then
else --[[continue]]

    return --done

end

local --[[id]] id
local id2 -- new id
    = .123, function( arg --[[number]] , ... --[[args]]) end

local a --[[a]],  b, c = id + --[[add]] id, nil --[[nothing]]

local function fn --[[function name]] ( p1, p2 -- p2
    , p3, ... ) end

for i = 10, 1, - 3 do -- comment
    continue --skip
end

repeat --[[nothing]]
    break --!
until --[[condition]] not --[[value]] -value

while --[[true]] true --
do
    return --[[result]] result . new --[[fn name]] ( value . field --
), object[ --[[open]] not key --[[close]] ]
end

object : method --[[call]] ({ key --[[]] = [[true]], [ true --[[key]]] = ( nil ) -- nil
 }, ... --[[forward args]] )

object . --[[get field]] field : method --
{ if --[[condition]]  value   then --[[true]]   ok   else --[[false]]  err, { --[[empty table]] }, --[[trailing comma]] }

local string = `-{ true }-{ object --[[ok]] }={ c + 8 }` -- interpolated string
