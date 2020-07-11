
export default class DateUtil {
  static shortMonths = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  static toShortFormat(date) {
    let now = new Date();
    if (now.getDate() === date.getDate() && now.getMonth() === date.getMonth() && now.getFullYear() === date.getFullYear()) {
      return DateUtil.toColonHourFormat(date);
    } else {
      return [DateUtil.numPad(date.getDate()), DateUtil.shortMonths[date.getMonth()], date.getFullYear()].join(' ');
    }
  }

  static toColonHourFormat(date) {
    return [DateUtil.numPad(date.getHours()), DateUtil.numPad(date.getMinutes()), DateUtil.numPad(date.getSeconds())].join(':');
  }

  static toHyphenDateFormat(date) {
    return [date.getFullYear(), DateUtil.numPad(date.getMonth()+1), DateUtil.numPad(date.getDate())].join('-');
  }

  static toGmtDiffFormat(date) {
    let diff = date.getTimezoneOffset();
    if (diff == 0) {
      return 'GMT';
    } else {
      let hours = DateUtil.numPad(Math.abs(Math.floor(diff/60)));
      let mins = DateUtil.numPad(Math.round(diff/60 % 1 * 10) / 10 * 60);
      let sign = (diff > 0) ? '-' : '+'; // a positive offset means GMT-XX according to http://www.w3schools.com/jsref/jsref_gettimezoneoffset.asp
      return `GMT${sign}${hours}:${mins}`;
    }
  }

  static toLongGmtFormat(date) {
    let datePart = DateUtil.toHyphenDateFormat(date);
    let hourPart = DateUtil.toColonHourFormat(date);
    let tzPart = DateUtil.toGmtDiffFormat(date);
    return [datePart, hourPart, tzPart].join(' ');
  }

  static addMonths(date, monthDiff) {
    const result = new Date(date);
    result.setMonth(result.getMonth() + monthDiff);
    return result;
  }

  static addDays(date, dayDiff) {
    const result = new Date(date);
    result.setDate(result.getDate() + dayDiff);
    return result;
  }

  static addHours(date, hourDiff) {
    const result = new Date(date);
    result.setHours(result.getHours() + hourDiff);
    return result;
  }

  static addMinutes(date, minDiff) {
    const result = new Date(date);
    result.setMinutes(result.getMinutes() + minDiff);
    return result;
  }

  static numPad(nb) {
    if (nb < 10) {
      return `0${nb}`;
    } else {
      return `${nb}`;
    }
  }
}
