#include "destiny_carbon.hpp"

int main() {
    destiny::Ballpark park(false);
    const auto ball = park.AddBall(
        9, 1.0, 2.0, 100.0, true, false, true, true, false,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0);
    return ball.id() == 9 ? 0 : 1;
}
