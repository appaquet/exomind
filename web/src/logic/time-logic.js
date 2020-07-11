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
      date.setDate(date.getDate()+6-date.getUTCDay()); // saturday
      date.setHours(8);
      break;
    case 'next_week':
      date.setDate(date.getDate()+(7-date.getUTCDay()+1)); // monday 8h
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
