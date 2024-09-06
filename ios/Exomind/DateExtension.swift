import Foundation
import SwiftProtobuf

// https://github.com/justinmakaila/NSDate-ISO-8601/blob/master/NSDateISO8601.swift
// https://github.com/al7/SwiftDateExtension/blob/master/DateExtension/DateExtension.swift

public extension Date {
    fileprivate static let isoDateDeserializer: DateFormatter = {
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone.autoupdatingCurrent
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSZ"
        return dateFormatter
    }()

    fileprivate static let isoDateSerializer: DateFormatter = {
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(abbreviation: "GMT")
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS"
        return dateFormatter
    }()

    fileprivate static let shortTimeFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return formatter
    }()

    fileprivate static let shortDateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        return formatter
    }()

    static func ISOStringFromDate(_ date: Date) -> String {
        return isoDateSerializer.string(from: date) + "Z"
    }

    static func dateFromISOString(_ string: String) -> Date? {
        return isoDateDeserializer.date(from: string)
    }

    var millisecondsSince1970: Int64 {
        Int64((self.timeIntervalSince1970 * 1000.0).rounded())
    }

    init(milliseconds: Int64) {
        self = Date(timeIntervalSince1970: TimeInterval(milliseconds) / 1000)
    }

    func toShort() -> String {
        let midnight = Date().midnightUTCDate()
        if (self.isGreaterThan(midnight)) {
            return Date.shortTimeFormatter.string(from: self)
        } else {
            return Date.shortDateFormatter.string(from: self)
        }
    }

    func toLongFormat() -> String {
        return Date.shortDateFormatter.string(from: self) + " " + Date.shortTimeFormatter.string(from: self)
    }

    func plusSeconds(_ s: UInt) -> Date {
        return self.addComponentsToDate(seconds: Int(s), minutes: 0, hours: 0, days: 0, weeks: 0, months: 0, years: 0)
    }

    func minusSeconds(_ s: UInt) -> Date {
        return self.addComponentsToDate(seconds: -Int(s), minutes: 0, hours: 0, days: 0, weeks: 0, months: 0, years: 0)
    }

    func plusMinutes(_ m: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: Int(m), hours: 0, days: 0, weeks: 0, months: 0, years: 0)
    }

    func minusMinutes(_ m: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: -Int(m), hours: 0, days: 0, weeks: 0, months: 0, years: 0)
    }

    func plusHours(_ h: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: Int(h), days: 0, weeks: 0, months: 0, years: 0)
    }

    func minusHours(_ h: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: -Int(h), days: 0, weeks: 0, months: 0, years: 0)
    }

    func plusDays(_ d: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: Int(d), weeks: 0, months: 0, years: 0)
    }

    func minusDays(_ d: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: -Int(d), weeks: 0, months: 0, years: 0)
    }

    func plusWeeks(_ w: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: 0, weeks: Int(w), months: 0, years: 0)
    }

    func minusWeeks(_ w: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: 0, weeks: -Int(w), months: 0, years: 0)
    }

    func plusMonths(_ m: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: 0, weeks: 0, months: Int(m), years: 0)
    }

    func minusMonths(_ m: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: 0, weeks: 0, months: -Int(m), years: 0)
    }

    func plusYears(_ y: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: 0, weeks: 0, months: 0, years: Int(y))
    }

    func minusYears(_ y: UInt) -> Date {
        return self.addComponentsToDate(seconds: 0, minutes: 0, hours: 0, days: 0, weeks: 0, months: 0, years: -Int(y))
    }

    fileprivate func addComponentsToDate(seconds sec: Int, minutes min: Int, hours hrs: Int, days d: Int, weeks wks: Int, months mts: Int, years yrs: Int) -> Date {
        var dc: DateComponents = DateComponents()
        dc.second = sec
        dc.minute = min
        dc.hour = hrs
        dc.day = d
        dc.weekOfYear = wks
        dc.month = mts
        dc.year = yrs
        return (Calendar.current as NSCalendar).date(byAdding: dc, to: self, options: [])!
    }

    func midnightUTCDate() -> Date {
        var dc: DateComponents = (Calendar.current as NSCalendar).components([.year, .month, .day], from: self)
        dc.hour = 0
        dc.minute = 0
        dc.second = 0
        dc.nanosecond = 0
        (dc as NSDateComponents).timeZone = TimeZone(secondsFromGMT: 0)

        return Calendar.current.date(from: dc)!
    }

    static func secondsBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.second, from: d1, to: d2, options: [])
        return dc.second!
    }

    static func minutesBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.minute, from: d1, to: d2, options: [])
        return dc.minute!
    }

    static func hoursBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.hour, from: d1, to: d2, options: [])
        return dc.hour!
    }

    static func daysBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.day, from: d1, to: d2, options: [])
        return dc.day!
    }

    static func weeksBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.weekOfYear, from: d1, to: d2, options: [])
        return dc.weekOfYear!
    }

    static func monthsBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.month, from: d1, to: d2, options: [])
        return dc.month!
    }

    static func yearsBetween(date1 d1: Date, date2 d2: Date) -> Int {
        let dc = (Calendar.current as NSCalendar).components(NSCalendar.Unit.year, from: d1, to: d2, options: [])
        return dc.year!
    }

    //MARK- Comparison Methods

    func isGreaterThan(_ date: Date) -> Bool {
        return (self.compare(date) == .orderedDescending)
    }

    func isLessThan(_ date: Date) -> Bool {
        return (self.compare(date) == .orderedAscending)
    }

    //MARK- Computed Properties

    var day: UInt {
        return UInt((Calendar.current as NSCalendar).component(.day, from: self))
    }

    var month: UInt {
        return UInt((Calendar.current as NSCalendar).component(.month, from: self))
    }

    var year: UInt {
        return UInt((Calendar.current as NSCalendar).component(.year, from: self))
    }

    var hour: UInt {
        return UInt((Calendar.current as NSCalendar).component(.hour, from: self))
    }

    var minute: UInt {
        return UInt((Calendar.current as NSCalendar).component(.minute, from: self))
    }

    var second: UInt {
        return UInt((Calendar.current as NSCalendar).component(.second, from: self))
    }

    static func -(lhs: Date, rhs: Date) -> TimeInterval {
        lhs.timeIntervalSinceReferenceDate - rhs.timeIntervalSinceReferenceDate
    }

    func toProtobuf() -> Google_Protobuf_Timestamp {
        Google_Protobuf_Timestamp(date: self)
    }
}
