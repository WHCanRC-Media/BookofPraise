melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'2 c4 c4 d4 c4 c2 a2 \break

  % Line 2
  bes2 a4 g2 f2 e4 f2 \break

  % Line 3
  c'2 c4 c4 d4 c4 c2 a2 \break

  % Line 4
  bes2 a4 g2 f2 e4 f2 \break

  % Line 5
  r4 a4 g4 e4 f4 d4 c2 \break

  % Line 6
  r4 c'4 c4 c4 d4 c4 c2 a2 \break

  % Line 7
  bes2 a4 g2 f2 e4 f1 \bar "|."
}
