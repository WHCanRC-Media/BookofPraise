melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 bes2 bes2 a4 a4 bes4 c4 d4 c4 bes2 a2 r2 f2 g4 b4 a2 c2 b4 b4 a4 g4 a2 g2 r2 g2 b4 b4 a2 g2 f4 g4 f4 e4 d2 r2 \break

  % Line 2
  d2 f4 f4 g2 b2 a4 g4 g4 fis4 g2 r2 \break

  % Line 3
  d2 c4 b4 a2 f2 g4 a4 b4 d4 c2 b2 r2 \break

  % Line 4
  b2 a4 g4 c2 d2 c4 b4 a4 g4 a2 g2 r2 \break

  % Line 5
  b2 a4 g4 f2 g2 f4 g4 f4 e4 d2 r2 \break

  % Line 6
  f2 g4 a4 b2 c2 b4 b4 a4 a4 g1 \bar "|."
}
