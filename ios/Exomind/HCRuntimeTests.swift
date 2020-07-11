//
//  HCRuntimeTests.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-07-31.
//  Copyright © 2016 Exomind. All rights reserved.
//

import XCTest
import SwiftyJSON

class HCRuntimeTests: XCTestCase {

    override func setUp() {
        super.setUp()
        self.continueAfterFailure = false
        HCNamespaces.registerNamespace(ExomindNamespace())
    }

    func testToBuilder() {
        let contact1 = ContactFull(email: "bob1@gmail.com", name: "Bob1")
        let contact2 = ContactFull(email: "bob2@gmail.com")
        let source = IntegrationSourceFull(data: [:], integrationKey: "bob@gmail.com", integrationName: "gmail")
        let email = EmailSummary(bcc: [], cc: [], from: contact1, id: "id1", receivedDate: Date(), source: source, to: [contact1, contact2])

        let builder = email.toBuilder()
        XCTAssertNil(builder.error)

        let emailDeser = builder.buildSummary() as? EmailSummary
        XCTAssertNotNil(emailDeser)
    }

    func testSimpleCompare() {
        let contact1 = ContactFull(email: "bob1@gmail.com", name: "Bob1")
        let contact2 = ContactFull(email: "bob2@gmail.com")

        XCTAssertEqual(2, contact1.diff(contact2).count)
        XCTAssertEqual(0, contact1.diff(contact1).count)
    }

    func testComplexeCompare() {
        let date = Date()
        let contact1 = ContactFull(email: "bob1@gmail.com", name: "Bob1")
        let contact2 = ContactFull(email: "bob2@gmail.com")
        let source = IntegrationSourceFull(data: [:], integrationKey: "bob@gmail.com", integrationName: "gmail")
        let email1 = EmailSummary(bcc: [], cc: [], from: contact1, id: "id1", receivedDate: date, source: source, to: [contact1, contact2])
        XCTAssertEqual(0, email1.diff(email1).count)

        let email2 = EmailSummary(bcc: [], cc: [], from: contact1, id: "id1", receivedDate: date, source: source, to: [contact1, contact1], modificationDate: Date())
        XCTAssertEqual(2, email2.diff(email1).count)
    }
    
    func testArray1Compare() {
        let contact1 = ContactFull(email: "bob1@gmail.com")
        let contact2 = ContactFull(email: "bob2@gmail.com")
        let email1 = DraftEmailFull(attachments: [], bcc: [], cc: [], parts: [], to: [contact1])
        let email2 = DraftEmailFull(attachments: [], bcc: [], cc: [], parts: [], to: [contact2])
        XCTAssertEqual(1, email2.diff(email1).count)
    }
    
    func testArray2Compare() {
        let contact1 = ContactFull(email: "bob1@gmail.com", name: "Bob1")
        let source = IntegrationSourceFull(data: [:], integrationKey: "bob@gmail.com", integrationName: "gmail")
        
        let part1full = EmailPartHtmlFull(body: "body1")
        let part1summary = EmailPartHtmlSummary()
        let date = Date()
        let email1 = EmailFull(attachments: [], bcc: [], cc: [], from: contact1, id: "id1", parts: [part1full], receivedDate: date, source: source, to: [contact1])
        let email2 = EmailFull(attachments: [], bcc: [], cc: [], from: contact1, id: "id1", parts: [part1summary], receivedDate: date, source: source, to: [contact1])
        let email3 = EmailSummary(bcc: [], cc: [], from: contact1, id: "id1", receivedDate: date, source: source, to: [contact1])

        XCTAssertEqual(1, email1.diff(email2).count)
        XCTAssertEqual(0, email1.diff(email1).count)
        XCTAssertEqual(1, email1.diff(email3).count)
    }

    func testClone() {
        let contact1 = ContactFull(email: "bob1@gmail.com", name: "Bob1")
        let contact2 = ContactFull(email: "bob2@gmail.com")
        let source = IntegrationSourceFull(data: [:], integrationKey: "bob@gmail.com", integrationName: "gmail")
        let email1 = EmailSummary(bcc: [], cc: [], from: contact1, id: "id1", receivedDate: Date(), source: source, to: [contact1, contact2])

        let email2 = email1.clone() as! EmailSummary
        XCTAssertTrue(email1.equals(email2))

        email2.subject = "Hello"
        XCTAssertFalse(email1.equals(email2))
    }
}
