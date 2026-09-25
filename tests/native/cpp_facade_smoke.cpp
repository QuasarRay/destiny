#include "destiny_carbon.hpp"

int main() {
    destiny::Ballpark park(false);
    const auto missing = park.GetCenterDist(101, 202);
    if (missing.has_value()) return 1;
    const auto ball = park.AddBall(
        101, 1.0, 2.0, 100.0, true, false, true, true, false,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0);
    return ball.id() == 101 ? 0 : 2;
}
