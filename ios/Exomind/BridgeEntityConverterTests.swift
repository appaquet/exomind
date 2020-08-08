
import XCTest
import SwiftyJSON
import JavaScriptCore

class BridgeEntityConverterTests: XCTestCase {
    override func setUp() {
        super.setUp()
        self.continueAfterFailure = false
        DomainStoreTests.setupInstance()
    }

    func testDeserializationSimpleStructure() {
        let note1 = NoteSummary(id: "note1", title: "Title1")
        let entity = HCEntity(id: "id1", traits: [note1])

        let jsObject = BridgeEntityConverter.entityToJavascript(entity)
        XCTAssertNil(BridgeEntityConverter.hcSerializer.error)
        XCTAssertNotNil(jsObject)

        let deserEntity = BridgeEntityConverter.entityFromJavascript(jsObject!)
        XCTAssertNil(BridgeEntityConverter.hcSerializer.error)
        XCTAssertNotNil(deserEntity)

        XCTAssertEqual(1, deserEntity?.traits.count)
        let note = deserEntity!.traits[0] as! Note
        XCTAssertEqual("Title1", note.title)
    }
}
