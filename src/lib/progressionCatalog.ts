export type ProgressionTitle = {
  minLevel: number;
  maxLevel: number;
  name: string;
  description: string;
  flavor?: string;
};

const ROYAL_CONTRACT_ID = "royal_contract";
const CONTRACT_UPGRADED_TITLE = "Court Marketing Manager";

export const progressionTitles: ProgressionTitle[] = [
  {
    minLevel: 1,
    maxLevel: 1,
    name: "Some Noob",
    description: "New to the Aethernet Kingdom",
  },
  {
    minLevel: 2,
    maxLevel: 5,
    name: "Court Marketing Intern",
    description: "Fresh recruit in the royal influence office",
    flavor: "Still learning which end of the slogan is business-facing",
  },
  {
    minLevel: 6,
    maxLevel: 10,
    name: "Assistant Royal Page",
    description: "Trusted with minor court messaging tasks",
  },
  {
    minLevel: 11,
    maxLevel: 15,
    name: "Junior Campaign Coordinator",
    description: "Allowed to schedule goblin focus groups",
  },
  {
    minLevel: 16,
    maxLevel: 20,
    name: "Court Communications Clerk",
    description: "Officially part of the royal influence office",
    flavor: "Your signature can now delay a proclamation",
  },
  {
    minLevel: 21,
    maxLevel: 25,
    name: "Senior Court Communications Clerk",
    description: "Knows which announcements should never be archived",
    flavor: "You know which forms can make scandals disappear",
  },
  {
    minLevel: 26,
    maxLevel: 30,
    name: "Royal Marketing Associate",
    description: "First taste of managing the crown's reputation",
    flavor: "You have acquired a clipboard and the confidence to brief a duke",
  },
  {
    minLevel: 31,
    maxLevel: 35,
    name: "Campaign Supervisor",
    description: "Managing interns and minor royal campaigns",
    flavor: "The interns fear your campaign calendars",
  },
  {
    minLevel: 36,
    maxLevel: 40,
    name: "Regional Influence Captain",
    description: "Responsible for royal reputation in distant provinces",
    flavor: "Your jurisdiction now includes villages that spell Aethernet differently",
  },
  {
    minLevel: 41,
    maxLevel: 45,
    name: "Royal Influence Officer",
    description: "Focused on growing the royal court's reputation",
    flavor: "Reputation is just logistics with better heraldry",
  },
  {
    minLevel: 46,
    maxLevel: 50,
    name: "Court Marketing Director",
    description: "A respected leader in the royal influence machine",
    flavor: "Your calendar is now more dangerous than most coup attempts",
  },
  {
    minLevel: 51,
    maxLevel: 55,
    name: "Chief Campaign Strategist",
    description: "Plans kingdom-wide royal messaging campaigns",
    flavor: "You can turn a vague prophecy into a royal rollout plan",
  },
  {
    minLevel: 56,
    maxLevel: 60,
    name: "Master of Royal Operations",
    description: "Runs the court's day-to-day influence machine",
    flavor: "If something moves, you have already scheduled it",
  },
  {
    minLevel: 61,
    maxLevel: 65,
    name: "High Chancellor of Messaging",
    description: "Now shaping the crown's official voice",
    flavor: "Royalty has discovered your inbox and the kingdom will hear about it",
  },
  {
    minLevel: 66,
    maxLevel: 70,
    name: "Grand Marshal of Campaigns",
    description: "Commands the royal court's largest influence initiatives",
    flavor: "Your campaign map requires a cartographer and a legal review",
  },
  {
    minLevel: 71,
    maxLevel: 75,
    name: "Royal Brand Advisor",
    description: "The crown starts taking your positioning seriously",
    flavor: "The King has started forwarding your taglines",
  },
  {
    minLevel: 76,
    maxLevel: 80,
    name: "Archchancellor of Aethernet Reach",
    description: "One of the most influential messengers in the kingdom",
    flavor: "Your memos travel faster than most spells and trend harder than dragons",
  },
  {
    minLevel: 81,
    maxLevel: 85,
    name: "Supreme Court Marketer",
    description: "The royal court's influence office is now legendary",
    flavor: "Bards have started quoting your campaign briefs",
  },
  {
    minLevel: 86,
    maxLevel: 90,
    name: "Lord of Royal Influence",
    description: "Your recommendations shape kingdom culture",
    flavor: "A casual suggestion from you becomes policy by lunch",
  },
  {
    minLevel: 91,
    maxLevel: 95,
    name: "Aetherlord of Messaging",
    description: "A title granted to only a handful of royal operators in history",
    flavor: "Entire markets panic when the court account posts",
  },
  {
    minLevel: 96,
    maxLevel: 99,
    name: "Legend of the Royal Court",
    description: "People know your campaigns everywhere",
  },
  {
    minLevel: 100,
    maxLevel: 100,
    name: "Kingmaker",
    description:
      "Not the monarch. The person who decides what the kingdom believes about the monarch",
    flavor: "Influence has surpassed authority",
  },
];

export function titleForLevel(level: number) {
  return (
    progressionTitles.find(
      (progressionTitle) =>
        level >= progressionTitle.minLevel && level <= progressionTitle.maxLevel,
    ) ?? progressionTitles[progressionTitles.length - 1]
  );
}

export function titleForLevelAndInventory(level: number, inventoryItemIds: string[]) {
  const baseTitle = titleForLevel(level);

  if (
    baseTitle.name === "Court Marketing Intern" &&
    inventoryItemIds.includes(ROYAL_CONTRACT_ID)
  ) {
    return {
      ...baseTitle,
      name: CONTRACT_UPGRADED_TITLE,
      description: "Official marketing manager for the Aethernet royal court",
      flavor: "The contract is mostly legitimate, depending on who asks",
    };
  }

  return baseTitle;
}

export function isPromotionLevel(level: number) {
  if (level <= 1) {
    return false;
  }

  return titleForLevel(level).name !== titleForLevel(level - 1).name;
}

export function titleHoverText(progressionTitle: ProgressionTitle) {
  return progressionTitle.flavor ?? progressionTitle.description;
}
