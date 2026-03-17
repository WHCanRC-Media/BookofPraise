melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 f2 g2 a2 d2 c4 b4 a4 c4 b2 a2 r2 \break

  % Line 2
  d2 d4 d4 c2 e2 d4 a4 bes4 a4 g2 f2 r2 \break

  % Line 3
  f2 e4 e4 d2 a'2 c4 c4 g2 bes2 a2 \break

  % Line 4
  r4 a4 c2 d2 a2 c2 a4 g4 f2 e2 d1 \bar "|."
}
