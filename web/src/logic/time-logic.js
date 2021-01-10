import DateUtil from '../utils/date-util.js';

export default class TimeLogic {
  static MaxEpoch = 32506380061000;

  static getLaterChoices() {
    return [
      {key: 'evening', copy: 'This Evening'},
      {key: 'later_today', copy: 'Later Today'},
      {key: 'next_morning', copy: 'Next Morning'},
      {key: 'next_evening', copy: 'Next Evening'},
      {key: 'weekend', copy: 'This Weekend'},
      {key: 'next_week', copy: 'Next Week'},
      {key: 'next_month', copy: 'Next Month'},
      {key: 'pick', copy: 'Pick'}
    ];
  }

  static getLaterIcon(key) {
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
    case 'pick':
      return 'calendar';
    }
  }

  static textDiffToDate(textDiff) {
    var date = new Date();
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
      if (date.getUTCDay() < 6) {
        date = DateUtil.addDays(date, 6 - date.getUTCDay()); // this saturday
      } else {
        date = DateUtil.addDays(date, 7); // we're saturday, so next one
      }
      date.setHours(8);
      break;
    case 'next_week':
      if (date.getUTCDay() < 1) {
        date = DateUtil.addDays(date, 1); // we're sunday, so tomorrow
      } else {
        date = DateUtil.addDays(date, 8 - date.getUTCDay()); // next monday
      }
      date.setHours(8);
      break;
    case 'next_month':
      date = DateUtil.addMonths(date, 1);
      date.setDate(1);
      date.setHours(8);
      break;
    }
    return date;
  }

}
