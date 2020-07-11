//
//  HCQueriesTests.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-07-31.
//  Copyright © 2016 Exomind. All rights reserved.
//

import XCTest
import SwiftyJSON

class HCQueriesTests: XCTestCase {

    override func setUp() {
        super.setUp()
        self.continueAfterFailure = false
    }

    func testSimple() {
        let json1 = HCQueries.Entities().withTrait("some_trait").toJSON()
        print(json1)
        let json2 = HCQueries.Entities().withTrait("some_trait") {
            b in
            b.refersTo("bob")
        }.toJSON()
        print(json2)
    }

    func testHashing() {
        let hash1 = HCQueries.Entities().withTrait("test").hash()
        let hash2 = HCQueries.Entities().withTrait("some_trait") {
            b in
            b.refersTo("bob")
        }.hash()
        XCTAssertNotEqual(hash1, hash2)
    }
}
