const markChosen = (event) => {
  event.currentTarget.dataset.chosen = "";
};
const chosenItem = {
  onClick: markChosen,
  onKeyDown: (event) => {
    if (event.key === "Enter" || event.key === " ") markChosen(event);
  }
};
export {
  chosenItem
};
