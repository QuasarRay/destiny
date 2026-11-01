// Test-only harness for the ACTUAL constructor from src/Partitionable.cpp.
// The Python runner extracts and injects the original initializer-list body.
// Only the unrelated external Blue Vector3 type is stubbed for compilation.
#include <cstdint>
#include <cstddef>
#include <cmath>
#include <iostream>
#include <type_traits>
struct Vector3 {
    float x,y,z;
    Vector3(float a, float b, float c) : x(a), y(b), z(c) {}
};
#include "src/Partitionable.h"

static_assert(std::is_abstract_v<Partitionable>, "C++ source is an abstract class");
static_assert(std::is_same_v<ID, std::int64_t>, "ID typedef changed");

// __ORIGINAL_PARTITIONABLE_CONSTRUCTOR__

// A concrete test-only subclass solely to instantiate the abstract base.
// The virtual methods are placeholders and are NEVER used as oracles.
struct ConstructorWitness : Partitionable {
    void ClearBoxes() override {}
    void DeleteFromBoxes() override {}
    void InsertInBox(Box*) override {}
    float GetBoundingRadius() override { return 0.0f; }
    double GetInflatedRadius(double) override { return 0.0; }
    void SetBoxActive(int, bool) override {}
    void InsertInBoxes(Box*, Box*, long) override {}
};
int main() {
    ConstructorWitness p;
    std::cout << "state "
        << p.mId << " " << static_cast<int>(p.mHandled) << " "
        << p.mNewBubble << " " << p.mOldBubble << " "
        << static_cast<int>(p.isInteractive) << " "
        << p.mBoxes.size() << " "
        << static_cast<int>(p.mMyBox == nullptr) << " "
        << static_cast<int>(p.mPark == nullptr) << " "
        << p.mEffectStamp << " " << static_cast<int>(p.isMoribund) << "\n";
    std::cout << "vectors "
        << p.mNewPos.x << " " << p.mNewPos.y << " " << p.mNewPos.z << " "
        << p.mOldPos.x << " " << p.mOldPos.y << " " << p.mOldPos.z << "\n";
    return 0;
}
