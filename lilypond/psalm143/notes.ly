melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 a'2 a2 g4 a4 b4 c4 b2 a2 r2 \break

  % Line 2
  d2 d4 d4 c4 b4 a4 c4 b2 a2 r2 \break

  % Line 3
  d2 a'4 a4 g4 a4 c2 b2 a2 r2 \break

  % Line 4
  d2 d4 d4 c4 b4 a4 c4 b2 a2 r2 \break

  % Line 5
  c2 b4 a4 g4 f4 g2 e2 d1 \bar "|."
}
