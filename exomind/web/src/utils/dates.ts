import { google } from "../protos";

export type SnoozeKey =
  'evening' | 'later_today' | 'next_morning' | 'next_evening' |
  'weekend' | 'next_week' | 'next_month' | 'pick' | 'plus_1_week' |
  'plus_2_weeks' | 'plus_4_weeks' | 'plus_3_months';

export interface ISnoozeChoice {
  key: SnoozeKey;
  copy: string;
}

export default class DateUtil {
  static MaxEpoch = 32506380061000;
  static shortMonths = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  // Exposed to iOS
  static getSnoozeChoices(): ISnoozeChoice[] {
    return [
      { key: 'evening', copy: 'This evening' },
      { key: 'later_today', copy: 'Later today' },
      { key: 'next_morning', copy: 'Next morning' },
      { key: 'next_evening', copy: 'Next evening' },
      { key: 'weekend', copy: 'Next weekend' },
      { key: 'next_week', copy: 'Next week' },
      { key: 'next_month', copy: 'Next month' },
      { key: 'plus_1_week', copy: '~1 week' },
      { key: 'plus_2_weeks', copy: '~2 weeks' },
      { key: 'plus_4_weeks', copy: '~4 weeks' },
      { key: 'plus_3_months', copy: '~3 months' },
      { key: 'pick', copy: 'Pick' },
    ];
  }

  // Exposed to iOS
  static getSnoozeIcon(key: SnoozeKey): string {
    switch (key) {
      case 'evening':
        return 'moon-o';
      case 'later_today':
        return 'hourglass-start';
      case 'next_morning':
        return 'coffee';
      case 'next_evening':
        return 'chevron-right';
      case 'weekend':
        return 'soccer-ball-o';
      case 'next_week':
        return 'briefcase';
      case 'next_month':
        return 'calendar-plus-o';
      case 'plus_1_week':
        return 'plus-square';
      case 'plus_2_weeks':
        return 'plus-square';
      case 'plus_4_weeks':
        return 'plus-square';
      case 'plus_3_months':
        return 'calendar-plus-o';
      case 'pick':
        return 'calendar';
    }
  }

  // Exposed to iOS
  static snoozeDate(textDiff: SnoozeKey): Date {
    let date = new Date();
    date.setMinutes(0);
    date.setSeconds(0);

    switch (textDiff) {
      case 'evening':
        date.setHours(18);
        break;
      case 'later_today':
        date = DateUtil.addHours(date, 2);
        break;
      case 'next_morning':
        date = DateUtil.addDays(date, 1);
        date.setHours(8);
        break;
      case 'next_evening':
        date = DateUtil.addDays(date, 1);
        date.setHours(18);
        break;
      case 'weekend':
        if (date.getDay() < 6) {
          date = DateUtil.addDays(date, 6 - date.getDay()); // this saturday
        } else {
          date = DateUtil.addDays(date, 7); // we're saturday, so next one
        }
        date.setHours(8);
        break;
      case 'next_week':
        if (date.getDay() < 1) {
          date = DateUtil.addDays(date, 1); // we're sunday, so tomorrow
        } else {
          date = DateUtil.addDays(date, 8 - date.getDay()); // next monday
        }
        date.setHours(8);
        break;
      case 'next_month':
        date = DateUtil.addMonths(date, 1);
        date.setDate(1);
        date.setHours(8);
        break;
      case 'plus_1_week':
        date = DateUtil.addDays(date, DateUtil.rand(5, 9));
        date.setHours(8);
        break;
      case 'plus_2_weeks':
        date = DateUtil.addDays(date, DateUtil.rand(11, 19));
        date.setHours(8);
        break;
      case 'plus_4_weeks':
        date = DateUtil.addDays(date, DateUtil.rand(20, 40));
        date.setHours(8);
        break;
      case 'plus_3_months':
        date = DateUtil.addDays(date, DateUtil.rand(75, 115));
        date.setHours(8);
        break;
    }
    return date;
  }

  static rand(from: number, to: number): number {
    return Math.floor(Math.random() * (to - from + 1)) + from;
  }

  static toShortFormat(date: Date): string {
    const now = new Date();
    if (now.getDate() === date.getDate() && now.getMonth() === date.getMonth() && now.getFullYear() === date.getFullYear()) {
      return DateUtil.toColonHourFormat(date);
    } else {
      return [DateUtil.numPad(date.getDate()), DateUtil.shortMonths[date.getMonth()], date.getFullYear()].join(' ');
    }
  }

  static toLongFormat(date: Date): string {
    return this.toHyphenDateFormat(date) + ' ' + DateUtil.toColonHourFormat(date);
  }

  static toColonHourFormat(date: Date): string {
    return [DateUtil.numPad(date.getHours()), DateUtil.numPad(date.getMinutes()), DateUtil.numPad(date.getSeconds())].join(':');
  }

  static toHyphenDateFormat(date: Date): string {
    return [date.getFullYear(), DateUtil.numPad(date.getMonth() + 1), DateUtil.numPad(date.getDate())].join('-');
  }

  static toGmtDiffFormat(date: Date): string {
    const diff = date.getTimezoneOffset();
    if (diff == 0) {
      return 'GMT';
    } else {
      const hours = DateUtil.numPad(Math.abs(Math.floor(diff / 60)));
      const mins = DateUtil.numPad(Math.round(diff / 60 % 1 * 10) / 10 * 60);
      const sign = (diff > 0) ? '-' : '+'; // a positive offset means GMT-XX according to http://www.w3schools.com/jsref/jsref_gettimezoneoffset.asp
      return `GMT${sign}${hours}:${mins}`;
    }
  }

  static toLongGmtFormat(date: Date): string {
    const datePart = DateUtil.toHyphenDateFormat(date);
    const hourPart = DateUtil.toColonHourFormat(date);
    const tzPart = DateUtil.toGmtDiffFormat(date);
    return [datePart, hourPart, tzPart].join(' ');
  }

  static addMonths(date: Date, monthDiff: number): Date {
    const result = new Date(date);
    result.setMonth(result.getMonth() + monthDiff);
    return result;
  }

  static addDays(date: Date, dayDiff: number): Date {
    const result = new Date(date);
    result.setDate(result.getDate() + dayDiff);
    return result;
  }

  static addHours(date: Date, hourDiff: number): Date {
    const result = new Date(date);
    result.setHours(result.getHours() + hourDiff);
    return result;
  }

  static addMinutes(date: Date, minDiff: number): Date {
    const result = new Date(date);
    result.setMinutes(result.getMinutes() + minDiff);
    return result;
  }

  static numPad(nb: number): string {
    if (nb < 10) {
      return `0${nb}`;
    } else {
      return `${nb}`;
    }
  }
}


export function toProtoTimestamp(date: Date): google.protobuf.ITimestamp {
  const epoch = date.getTime();
  const seconds = Math.floor(epoch / 1000);

  return new google.protobuf.Timestamp({
      seconds: seconds,
      nanos: (epoch - (seconds * 1000)) * 1000000,
  });
}

export function fromProtoTimestamp(ts: google.protobuf.ITimestamp): Date {
  return new Date((ts.seconds as number) * 1000 + ts.nanos / 1000000);
}