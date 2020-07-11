//
//  HCJsonSerializationTests.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-07-10.
//  Copyright © 2016 Exomind. All rights reserved.
//

import XCTest
import SwiftyJSON

class HCJsonSerializationTests: XCTestCase {

    override func setUp() {
        super.setUp()
        self.continueAfterFailure = false
        HCNamespaces.registerNamespace(ExomindNamespace())
    }

    func testDeserializationSimpleStructure() {
        let serializer = HCJsonSerialization()

        let json: JSON = [
                "_type": "exomind.contact",
                "name": "Bob",
                "email": "bob@roger.com"
        ]

        let recordBuilder = serializer.deserializeBuilder(json)
        let contactBuilder = recordBuilder as! ContactBuilder
        XCTAssertEqual(contactBuilder.name!!, "Bob")
        XCTAssertEqual(contactBuilder.email!, "bob@roger.com")

        let contact = contactBuilder.build() as! ContactFull
        XCTAssertEqual(contact.name!, "Bob")
        XCTAssertEqual(contact.email, "bob@roger.com")

        let contactSummary = contactBuilder.buildSummary() as! ContactSummary
        XCTAssertEqual(contactSummary.name!, "Bob")
        XCTAssertEqual(contactSummary.email, "bob@roger.com")
    }

    func testDeserializationArrayStructure() {
        let serializer = HCJsonSerialization()

        let json: JSON = [
                "_type": "exomind.email",
                "from": [
                        "_type": "exomind.contact",
                        "email": "some@test.com"
                ],
                "to": [
                        [
                                "_type": "exomind.contact",
                                "email": "to@toto.com"
                        ]
                ],
                "source": [
                        "_type": "exomind.integration_source",
                        "integration_name": "google",
                        "integration_key": "some@gmail.com",
                        "data": [
                                "_": "_"
                        ]
                ],
                "bcc": [],
                "cc": [],
                "received_date": "2016-07-10T23:30:18.239Z",
                "attachments": [],
                "parts": [],
                "subject": "Some subject",
                "id": "someid"
        ]

        let recordBuilder = serializer.deserializeBuilder(json)
        XCTAssertNil(recordBuilder?.error)
        XCTAssertNil(serializer.error)
        XCTAssertNotNil(recordBuilder?.buildSummary())
        XCTAssertNotNil(recordBuilder?.build())

        let record = serializer.deserializeRecord(json) as? HCFullRecord
        XCTAssertNotNil(record)
    }

    func testSerializationSimple() {
        let serializer = HCJsonSerialization()

        let contact1 = ContactFull(email: "bob@toto.com", name: "test")
        let contact2 = ContactFull(email: "bob@toto.com")
        let source = IntegrationSourceFull(data: ["key": "value"], integrationKey: "google", integrationName: "key")
        let email = EmailSummary(bcc: [], cc: [], from: contact1, id: "id", receivedDate: Date(), source: source, to: [contact1, contact2])

        let json = serializer.serialize(email)
        XCTAssertNil(serializer.error)

        print(json)

        let record = serializer.deserializeRecord(json) as? EmailSummary
        XCTAssertNil(serializer.error)
        XCTAssertNotNil(record)

        let deserEmail = record!
        XCTAssertEqual("id", deserEmail.id)
    }
    
    func testSerializationEntities() {
        let serializer = HCJsonSerialization()
        
        let note1 = NoteFull(id: "id1", title: "Note 1")
        note1.traitId = "id1"
        let note2 = NoteFull(id: "id2", title: "Note 2")
        let entity = HCEntity(id: "1234", traits: [note1, note2])
        
        let json = serializer.serialize(entity)
        XCTAssertNil(serializer.error)
        
        let deserEntity = serializer.deserializeEntity(json)!
        XCTAssertNil(serializer.error)
        
        let note1deser = deserEntity.traitsById["id1"] as! Note
        XCTAssertEqual("Note 1", note1deser.title)
    }
    
    func testSerializationNestedRecords() {
        let serializer = HCJsonSerialization()
        
        let contact1 = ContactFull(email: "bob@toto.com")
        let source = IntegrationSourceFull(data: ["key": "value"], integrationKey: "google", integrationName: "key")
        let parts = [EmailPartHtmlFull(body: "body")] 
        let email1 = EmailFull(attachments: [], bcc: [], cc: [], from: contact1, id: "id", parts: parts, receivedDate: Date(), source: source, to: [])
        let entity = HCEntity(id: "1234", traits: [email1])

        let json = serializer.serialize(entity)
        XCTAssertNil(serializer.error)
        
        let deserEntity = serializer.deserializeEntity(json)!
        XCTAssertNil(serializer.error)
        
        XCTAssertEqual(1, (deserEntity.traits[0] as! EmailFull).parts.count)
    }
    
    func testSerializationBuilders() {
        let serializer = HCJsonSerialization()
        
        let note1 = NoteBuilder()
        note1.title = "yo"
        note1.content = "some content"
        let jsonDict = serializer.serialize(note1)
        XCTAssertEqual("some content", jsonDict["content"].string)
        XCTAssertEqual("yo", jsonDict["title"].string)
    }

}
