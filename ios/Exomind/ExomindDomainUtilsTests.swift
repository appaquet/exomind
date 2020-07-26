//
//  ExomindDomainUtilsTests.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-11-08.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation

import XCTest
import SwiftyJSON
import JavaScriptCore

class ExomindDomainUtilsTests: XCTestCase {
    
    override func setUp() {
        super.setUp()
        self.continueAfterFailure = false
        DomainStoreTests.setupInstance()
    }
    
    func testTraitInformation() {
        let inbox = SpecialSummary(name: "Inbox")
        let inboxInfo = TraitInformationOld(ofTrait: inbox)
        XCTAssertEqual("Inbox", inboxInfo.displayName)
        XCTAssertEqual("inbox", inboxInfo.icon)
        
        let draftEmail = DraftEmailSummary(creationDate: nil, modificationDate: nil, sendingDate: nil, sentDate: nil, subject: "subject")
        let draftInfo = TraitInformationOld(ofTrait: draftEmail)
        XCTAssertEqual("subject", draftInfo.displayName)
    }
    
    func testDominantTrait() {
        let inbox = SpecialSummary(name: "Inbox")
        let note = NoteSummary(id: "note1", title: "title1")
        let collection = CollectionSummary(name: "col1")
        
        let entity1 = HCEntity(id: "id1", traits: [note, inbox])
        let dominantTrait1 = EntityTraitOld.dominantTrait(entity: entity1)
        XCTAssertNotNil(dominantTrait1)
        XCTAssertTrue(inbox.equals(dominantTrait1!))
        
        let entity2 = HCEntity(id: "id2", traits: [collection, note])
        let dominantTrait2 = EntityTraitOld.dominantTrait(entity: entity2)
        XCTAssertNotNil(dominantTrait2)
        XCTAssertTrue(collection.equals(dominantTrait2!))
    }
    
    func testEntityTrait() {
        let inbox = SpecialSummary(name: "Inbox")
        let note = NoteSummary(id: "note1", title: "title1")
    
        let entity = HCEntity(id: "id1", traits: [note, inbox])
        let entityTrait = EntityTraitOld(entity: entity)
        XCTAssertNotNil(entityTrait)
        XCTAssertEqual("Inbox", entityTrait?.displayName)
    }
    
}
