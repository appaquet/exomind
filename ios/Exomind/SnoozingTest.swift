import XCTest

class SnoozingTest: XCTestCase {
    override func setUpWithError() throws {
        JSBridgeTests.setupInstance()
    }

    func testLaterChoices() throws {
        XCTAssertGreaterThan(Snoozing.getLaterChoices().count, 5)
    }
}
