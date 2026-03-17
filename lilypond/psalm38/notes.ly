melody = \fixed c {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 bes'2 a'2 g'4 a'4 bes'4 c''4 d''2 c''2 \break

  % Line 2
  r4 d''2 c''4 bes'2 a'2 r2 \break

  % Line 3
  c''4 c''4 d''2 bes'4 c''4 a'2 g'2 r2 \break

  % Line 4
  d''2 d''2 c''4 bes'4 g'4 a'4 bes'2 a'2 \break

  % Line 5
  r4 bes'2 a'4 g'2 f'2 r2 \break

  % Line 6
  bes'2 d''2 c''4 bes'4 g'2 a'2 g'1 \bar "|."
}
