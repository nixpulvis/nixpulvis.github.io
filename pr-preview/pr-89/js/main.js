// Every fortune, for `fortune()`. Gathered here once, so pages like 404.md
// can share them rather than inlining their own copy.
const fortunes = ["It’s about time we till the pasture.","I bought you black roses for your birthday.\nBut you still can’t see.\nI brought you four more flowers.\nBut your freckles got in the way.","Wallpaint like jawbreakers. Seemingly endless, but shrinking.","Loose cannons aim for bells.","TLA* != TLA+","The _ were looking for a way to _ when they _.","You can't knock something on itself.","Today's new diets are just food for thought.","Always first describe *what* must be done, then move on to *how* if necessary.","No, *you're* wearing my clothes!","I just want to squeeze you into juice\nand drink the concentrated pulp nectar\nuntil we're balanced level","Times are changing too fast. I leave and I never come back.","If family is your rock, good friends are like the trees and the ferns.","Unreasonable -> Unfair","Be patient but persistent","Subtle changes to perfection are interesting.","Design to the highest degree but target the lowest common denominator.","Practice good posture.","Don't let just for fun mean it's never done.","Idea #7289172\n\nBuy (in bulk) from Amazon and resell (even through themselves),\nReturn broken and invalid / wrong products,\n    and forward on the products to the customers.\nChallenge amazon to perform better,\nProfit off customer reliability, something amazon is known to be bad at.","Interests alone a friend don't they make.","Refine your definition\nDefine your reality","Life is a puzzle without borders, and success is that feeling you get when you find the next piece.","You can't even see me face, cause I'm way off in space.","Not all intelligences are smart.","Shadow\n\nHe leads and he follows.","Oh great One, how should I best address you!","The young happily build sand castles, knowing the tide is coming. The old have since become attached to the stone, forgetting what sand is made of.","Programming is like building sand castles.","Don't confuse ego with purpose.","Do as I say not as I do.\nBecause I'd say I do do as I say\nJust not all the time.\nLet your words betray you.","The movement will be behind me one day\nAs long as I keep walking\nand walk stronger.","Trust but verify.","Illogical statements carry strong impact when spoken truthfully.","Luck favors a prepared mind.\n\n- Hamming","I was put here, so I will not be put out","I'm a caretaker without a care at all.","Government should be transparent and invisible.","Go out with your vapor and come back with your gut.","Be different without definition.","What was once possible may not be, impossible also.","Everything is a little thing and the little things are everything.","Try as you might, nothing's ever quite right.","There exist true unifying principals underlying life and everything else, but\nthey must go unspoken. For the moment you give them names, they cease to be\nunifying, degrading into muddy waters. All other truths are mere\napproximations, no matter how precise.","Search can fail, scanning should not.","Sometimes it's easier to clean up a mess than to avoid making it.","Why stop now?","Take your head off, rinse, put it back on, repeat.","OCD: Order, Construction, Discipline","Cry as you will, a bugs a bug.","Wear watches when the seconds matter.","All abstractions are simplifications.","We used to fight over the important issues, now we fight for the importance\nof issues.","Know own, No one...","Do not make rules you are not willing to defend.","You're *just* wrong!","There's no limit to what can be taught to those willing to learn.","Don't fear the unknown, there's plenty of known to fear.","You're always yourself, forever and beyond.","If 2016-2019 has taught us anything it's that we may not be able to trust\neveryone will be sane and competent, but it's how we deal with it that defines\nus.","Nothing eats time like a terminal and some wine.","Better to be confused clearly, than clearly confused.","If you present me the same argument enough times, I'll eventually get irritated.","Doing things you do a lot can be fast.","Better take 'em down to the karmic apothecary.","Color emoji are defined as eldritch horrors from a cursed world.\n\t\t-- Yaniel","Haste makes explosive diarrhea","I just need to know the things I care about.","If nobody will laugh with me, I'll just laugh with myself.","A bear is a lot to bear.","I've never met a bare bear.","A door is a metaphor. Knock, bust, or find a key.","If everything was perfect I wouldn't be very happy at all.","If you write it, they will read.","Don't force wisdom, let it force you.","Be opinionated, not entrenched.","Solving problems like it's easy, if you don't like it try not to get queasy.","Throw a little flair in there.","They recite the rhyme without remembering the reason.","You can't have a bug in a feature that doesn't exisit.","I decide history."];

// A random fortune, like `fortune` itself.
function fortune() {
  return fortunes[Math.floor(Math.random() * fortunes.length)];
}

// For anyone who opens the console.
console.log("%cλ", "font: 48px serif;");
console.log("%s", fortune());

document.addEventListener("DOMContentLoaded", () => {
  // Toggle my hidden, ugly mug.
  document.getElementById("lambda").addEventListener("click", (e) => {
    document.getElementById("me").classList.toggle("hidden");
  });

  // Add a "cheat" code.
  Mousetrap.bind('up up down down left right left right b a enter', function() {
    window.location.href = "/pr-preview/pr-89/MTk3ODkK";
  });

  // Link each heading to itself, shown on hover by `.anchor` in style.css. A
  // page's title (`.title`, from the layouts) links to the page itself.
  // Headings can contain their own links, so rather than wrapping the heading
  // in a link, clicks elsewhere on it are forwarded to the anchor.
  const anchored = [];
  document.querySelectorAll(".content :is(h1, h2, h3, h4, h5, h6):is([id], .title)").forEach((heading) => {
    const anchor = document.createElement("a");
    anchor.className = "anchor";
    if (heading.id) {
      anchor.href = "#" + heading.id;
      anchor.setAttribute("aria-label", "Link to this section");
    } else {
      anchor.href = window.location.pathname;
      anchor.setAttribute("aria-label", "Link to this page");
    }
    anchor.textContent = "#";
    heading.prepend(anchor);
    heading.classList.add("anchored");
    anchored.push(heading);
    heading.addEventListener("click", (e) => {
      // Skip clicks on links, and clicks that end a text selection.
      if (e.target.closest("a") || !window.getSelection().isCollapsed) return;
      anchor.click();
    });
  });

  // Mark the headings whose anchor doesn't fit in the margin, as `.cramped`,
  // so style.css can move it into the heading. Measured with every anchor
  // back in the margin, all at once, and again as the window or an opened
  // `details` changes what's shown.
  const fit = () => {
    anchored.forEach((heading) => heading.classList.remove("cramped"));
    anchored
      .filter((heading) => heading.querySelector(":scope > .anchor").getBoundingClientRect().left < 0)
      .forEach((heading) => heading.classList.add("cramped"));
  };
  fit();
  window.addEventListener("resize", fit);
  document.addEventListener("toggle", fit, true);
});
